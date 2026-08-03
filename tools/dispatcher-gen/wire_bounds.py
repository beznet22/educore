#!/usr/bin/env python3
"""
wire_bounds — batch-wire CommandBounds impls into all crates.

For each crate:
1. Add `educore-dispatcher` to Cargo.toml [dependencies].
2. Add `use educore_core::ids::IdempotencyKey;` + `use educore_dispatcher::CommandBounds;`
   to src/commands.rs (or src/services.rs if commands live there).
3. Append auto-generated impl blocks (deduped vs existing impls).
4. Run `cargo build` to verify.

Caveats:
- Skips if crate already has dispatcher-gen imports.
- Skips impl blocks that duplicate an existing impl in the crate.
- Reports per-crate build status at the end.
"""
from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def log(msg: str) -> None:
    print(msg, flush=True)


def cargo_toml_has(toml: Path, dep: str) -> bool:
    return dep in toml.read_text()


def add_dispatcher_dep(toml: Path) -> bool:
    """Add `educore-dispatcher = { workspace = true }` to Cargo.toml dependencies.
    Returns True if changed."""
    if cargo_toml_has(toml, "educore-dispatcher"):
        return False
    src = toml.read_text()
    # Find first `educore-` dep line and insert after the block.
    # Pattern: [dependencies]\n<dep lines>
    new_src = re.sub(
        r"(\[dependencies\]\n)((?:[^\[]+\n)*)",
        lambda m: m.group(1) + "educore-dispatcher = { workspace = true }\n" + m.group(2),
        src,
        count=1,
    )
    toml.write_text(new_src)
    return True


def add_imports(rs: Path) -> bool:
    """Add IdempotencyKey + CommandBounds imports to commands.rs."""
    src = rs.read_text()
    changed = False
    if "use educore_dispatcher::CommandBounds" not in src and "use educore_dispatcher::CommandBounds;" not in src:
        # Insert after the first educore_core::tenant::TenantContext line
        new_src = re.sub(
            r"(use educore_core::tenant::TenantContext[^\n]*\n)",
            lambda m: m.group(1) + "use educore_core::ids::IdempotencyKey;\n"
            + "use educore_dispatcher::CommandBounds;\n",
            src,
            count=1,
        )
        if new_src != src:
            rs.write_text(new_src)
            src = new_src
            changed = True
    return changed


def append_bounds(rs: Path, bounds_template: Path, services_rs: Path) -> tuple[int, int]:
    """Append CommandBounds impls from template, skipping duplicates.
    Returns (appended, skipped)."""
    existing_src = rs.read_text()
    services_src = services_rs.read_text() if services_rs.exists() else ""
    combined = existing_src + "\n" + services_src
    # Find command names already implemented in the combined file
    existing_impls = set(re.findall(
        r"impl\s+(?:educore_dispatcher::)?CommandBounds\s+for\s+(\w+)\s*\{",
        combined,
    ))
    template = bounds_template.read_text()
    impls = re.findall(
        r"impl educore_dispatcher::CommandBounds for \w+ \{(?:[^}]|\}(?!\}))*\}\n",
        template,
    )
    appended = 0
    skipped = 0
    new_blocks = []
    for impl in impls:
        m = re.search(r"for\s+(\w+)\s*\{", impl)
        if not m:
            continue
        cmd_name = m.group(1)
        if cmd_name in existing_impls:
            skipped += 1
            continue
        new_blocks.append(impl)
        existing_impls.add(cmd_name)
        appended += 1
    if new_blocks:
        with rs.open("a") as f:
            f.write("\n// Wire_bounds — auto-appended CommandBounds impls\n\n")
            for block in new_blocks:
                f.write(block + "\n")
    return appended, skipped


def process_crate(crate_dir: Path) -> tuple[str, str, int, int, bool]:
    """Returns (crate, status, appended, skipped, cargo_ok)."""
    cargo_toml = crate_dir / "Cargo.toml"
    commands_rs = crate_dir / "src" / "commands.rs"
    services_rs = crate_dir / "src" / "services.rs"
    if not cargo_toml.exists():
        return (str(crate_dir), "skip:no-cargo-toml", 0, 0, True)

    # Find the template file
    crate_slug = "_".join(crate_dir.relative_to(ROOT).parts[1:3])
    bounds_template = ROOT / "tools" / "dispatcher-gen" / "templates" / f"{crate_slug}_bounds.rs"
    if not bounds_template.exists():
        return (str(crate_dir), "skip:no-template", 0, 0, True)

    if not commands_rs.exists() and not services_rs.exists():
        return (str(crate_dir), "skip:no-commands", 0, 0, True)

    target_rs = commands_rs if commands_rs.exists() else services_rs

    dep_added = add_dispatcher_dep(cargo_toml)
    imports_added = add_imports(target_rs) if commands_rs.exists() else False

    # If commands.rs doesn't exist but services.rs does, add imports to services.rs too.
    if not commands_rs.exists() and services_rs.exists():
        # Find commands in services.rs and add impls directly there
        services_src = services_rs.read_text()
        if "use educore_dispatcher::CommandBounds" not in services_src:
            new_src = re.sub(
                r"(use educore_core::tenant::TenantContext[^\n]*\n)",
                lambda m: m.group(1) + "use educore_core::ids::IdempotencyKey;\n"
                + "use educore_dispatcher::CommandBounds;\n",
                services_src,
                count=1,
            )
            if new_src != services_src:
                services_rs.write_text(new_src)

    # Append CommandBounds impls (deduped)
    appended, skipped = append_bounds(target_rs, bounds_template, services_rs)

    # Run cargo build for this crate
    pkg_name = crate_dir.parent.name + "-" + crate_dir.name
    # Actually get the package name from Cargo.toml
    cargo_src = cargo_toml.read_text()
    m = re.search(r'^name\s*=\s*"([^"]+)"', cargo_src, re.MULTILINE)
    if m:
        pkg_name = m.group(1)
    result = subprocess.run(
        ["cargo", "build", "-p", pkg_name],
        cwd=ROOT,
        capture_output=True,
        text=True,
        timeout=180,
    )
    cargo_ok = result.returncode == 0
    status = "OK" if cargo_ok else f"FAIL: {result.stderr[-200:]}"
    return (str(crate_dir), status, appended, skipped, cargo_ok)


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--only", help="Only process this crate, e.g. 'domains/cms'")
    ap.add_argument("--dry-run", action="store_true", help="Just show what would be done")
    args = ap.parse_args()

    crates = []
    for cargo_toml in ROOT.glob("crates/**/Cargo.toml"):
        if "target" in str(cargo_toml):
            continue
        if args.only and args.only not in str(cargo_toml.parent):
            continue
        crates.append(cargo_toml.parent)

    print(f"Processing {len(crates)} crates...")
    results = []
    for c in crates:
        rel = str(c.relative_to(ROOT))
        if args.dry_run:
            print(f"  would process {rel}")
            continue
        crate, status, appended, skipped, ok = process_crate(c)
        results.append((rel, status, appended, skipped, ok))
        flag = "✓" if ok else "✗"
        print(f"  {flag} {rel}: {status} (appended={appended}, skipped={skipped})", flush=True)

    if not args.dry_run:
        ok_count = sum(1 for r in results if r[4])
        total_appended = sum(r[2] for r in results)
        total_skipped = sum(r[3] for r in results)
        print(f"\nDone: {ok_count}/{len(results)} crates build clean")
        print(f"Total: {total_appended} impls appended, {total_skipped} skipped (duplicates)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
