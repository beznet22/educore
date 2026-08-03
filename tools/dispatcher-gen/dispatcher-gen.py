#!/usr/bin/env python3
"""
dispatcher-gen — mass-produce `dispatch_X` wrappers for service functions.

Scans all crates' `src/services.rs` files, extracts the `pub fn X<C, G>(...)`
signatures, and generates:

1. `impl educore_dispatcher::CommandBounds for XCommand { ... }` blocks.
2. `pub async fn dispatch_X<C, G>(...)` wrappers that call
   `CommandDispatcher::dispatch(&cmd, &[capability], || async { X(cmd, ...) })`.

This is the foundation for wrapping all 382 service functions across the
20 crates that have services. Template established in Wave 192.

Usage:
    python3 tools/dispatcher-gen/dispatcher-gen.py --domain academic
    python3 tools/dispatcher-gen/dispatcher-gen.py --all
    python3 tools/dispatcher-gen/dispatcher-gen.py --dry-run
"""
from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path
from typing import NamedTuple


ROOT = Path(__file__).resolve().parents[2]


class ServiceFn(NamedTuple):
    crate: str           # e.g. "domains/academic"
    name: str            # e.g. "admit_student"
    command_type: str    # e.g. "AdmitStudentCommand"
    is_async: bool       # True for pub async fn
    return_type: str     # e.g. "Result<(Student, StudentAdmitted)>"
    has_uniqueness: bool # True if signature has &dyn UniquenessChecker
    has_port: bool       # True if signature has &dyn SomePort


def scan_crate(crate_dir: Path) -> list[ServiceFn]:
    """Scan a crate's src/services.rs for pub fn signatures."""
    services = crate_dir / "src" / "services.rs"
    if not services.exists():
        return []
    src = services.read_text()
    crate = "/".join(crate_dir.relative_to(ROOT).parts[1:3])
    # Match: pub (async)? fn NAME<C, G>( ... ) -> RESULT_TYPE
    # where RESULT_TYPE may span multiple lines until -> or {
    pattern = re.compile(
        r"^pub\s+(async\s+)?fn\s+(\w+)\s*<C,\s*G>\s*\(([^)]*)\)\s*(?:->\s*([^{]+))?",
        re.MULTILINE | re.DOTALL,
    )
    out = []
    for m in pattern.finditer(src):
        is_async = m.group(1) is not None
        name = m.group(2)
        args = m.group(3)
        ret = (m.group(4) or "").strip()
        # Skip the wrapper itself
        if name.startswith("dispatch_"):
            continue
        # Skip pub fns that don't take a Command as first arg
        first_arg = args.split(",")[0].strip() if "," in args else args.strip()
        if not first_arg.endswith("Command"):
            continue
        cmd = first_arg.split()[-1]
        out.append(ServiceFn(
            crate=crate,
            name=name,
            command_type=cmd,
            is_async=is_async,
            return_type=ret,
            has_uniqueness="UniquenessChecker" in args,
            has_port=re.search(r"&dyn\s+\w+", args) is not None,
        ))
    return out


def to_snake_action(name: str) -> str:
    """admit_student -> 'admit'; update_student_profile -> 'update'."""
    parts = name.split("_")
    return parts[0]


def to_target_type(cmd: str) -> str:
    """AdmitStudentCommand -> 'student'; CreateClassCommand -> 'class'."""
    m = re.match(r"(\w+)Command$", cmd)
    if not m:
        return "unknown"
    name = m.group(1)
    # Strip trailing verbs: Admit/Update/Create/Delete/Set/Assign/...
    for verb in ("Admit", "Update", "Create", "Delete", "Set", "Assign",
                 "Register", "Retire", "Mark", "Unlink", "Link", "Promote",
                 "Graduate", "Suspend", "Reinstate", "Withdraw", "Transfer",
                 "Close", "Copy", "Reassign", "Unassign", "Swap", "Cancel",
                 "Open", "Renew", "Bind", "Approve", "Reject"):
        if name.startswith(verb):
            name = name[len(verb):]
            break
    # Singularize trailing 's' (e.g. Classes -> class)
    if name.endswith("ies"):
        name = name[:-3] + "y"
    elif name.endswith("es") and not name.endswith("ss"):
        name = name[:-2]
    elif name.endswith("s") and not name.endswith("ss"):
        name = name[:-1]
    # Lowercase first letter
    return name[:1].lower() + name[1:] if name else "unknown"


def to_capability(cmd: str, action: str) -> str:
    """AdmitStudentCommand -> 'academic.student.create'."""
    # Crude: infer domain from crate path
    return f"<inferred>.{to_target_type(cmd)}.{action}"


def generate_command_bounds(svc: ServiceFn) -> str:
    """Generate the CommandBounds impl block for the command."""
    cmd = svc.command_type
    action = to_snake_action(svc.name)
    target = to_target_type(cmd)
    return f"""impl educore_dispatcher::CommandBounds for {cmd} {{
    fn tenant(&self) -> &TenantContext {{ &self.tenant }}
    fn command_type(&self) -> &'static str {{ "<domain>.{target}.{action}" }}
    fn idempotency_key(&self) -> Option<IdempotencyKey> {{ None }}
    fn action(&self) -> &'static str {{ "{action}" }}
    fn target_type(&self) -> &'static str {{ "{target}" }}
}}
"""


def generate_wrapper(svc: ServiceFn) -> str:
    """Generate the dispatch_X wrapper function."""
    dispatch_name = f"dispatch_{svc.name}"
    cmd = svc.command_type
    extra_args = ""
    extra_args_pass = ""
    if svc.has_uniqueness:
        extra_args = ",\n    uniqueness: &dyn UniquenessChecker,"
        extra_args_pass = ",\n        uniqueness,"
    return f"""/// Dispatcher wrapper for [`{svc.name}`].
///
/// Mirrors the Wave 192 template: clones `cmd` inside the closure
/// to satisfy the borrow checker, then runs the full
/// `CommandDispatcher::dispatch` pipeline (RBAC → txn →
/// idempotency → service → outbox → audit → idempotency record
/// → bus publish).
pub async fn {dispatch_name}<C, G>(
    dispatcher: &CommandDispatcher,
    cmd: {cmd},
    clock: &C,
    ids: &G{extra_args}
) -> Result<{svc.return_type}>
where
    C: Clock + ?Sized + Send + Sync,
    G: IdGenerator + ?Sized + Send + Sync,
{{
    use educore_dispatcher::CommandBounds as _;
    dispatcher
        .dispatch(&cmd, &["<capability>"], || async {{
            {svc.name}::<C, G>(cmd{extra_args_pass})
        }})
        .await
}}
"""


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--all", action="store_true", help="Scan all crates")
    ap.add_argument("--domain", help="Scan a specific crate, e.g. 'academic'")
    ap.add_argument("--dry-run", action="store_true", help="Print stats only")
    args = ap.parse_args()

    # Find all crates with src/services.rs
    all_fns: list[ServiceFn] = []
    for cargo_toml in ROOT.glob("crates/**/Cargo.toml"):
        crate_dir = cargo_toml.parent
        fns = scan_crate(crate_dir)
        if fns:
            all_fns.extend(fns)

    if args.domain:
        all_fns = [f for f in all_fns if args.domain in f.crate]

    # Group by crate
    by_crate: dict[str, list[ServiceFn]] = {}
    for f in all_fns:
        by_crate.setdefault(f.crate, []).append(f)

    print(f"=== {len(by_crate)} crates, {len(all_fns)} service fns ===\n")
    for crate, fns in sorted(by_crate.items()):
        print(f"  {crate}: {len(fns)} fns")
        for f in fns[:3]:
            print(f"    {f.name} ({f.command_type})")
        if len(fns) > 3:
            print(f"    ... +{len(fns) - 3} more")

    if args.dry_run:
        return 0

    # Generate a manifest
    manifest_path = ROOT / "tools" / "dispatcher-gen" / "manifest.md"
    lines = ["# Dispatcher Wrapper Manifest", "",
             "Generated by `tools/dispatcher-gen/dispatcher-gen.py`.",
             "",
             "| # | Crate | Service fn | Command | Wrapper |",
             "|---|-------|------------|---------|---------|"]
    idx = 0
    for crate, fns in sorted(by_crate.items()):
        for f in fns:
            idx += 1
            lines.append(
                f"| {idx} | `{crate}` | `{f.name}` | `{f.command_type}` | "
                f"`dispatch_{f.name}` |"
            )
    lines.append("")
    lines.append(f"**Total: {idx} wrappers needed.**")
    lines.append("")
    lines.append("Generated templates are in `tools/dispatcher-gen/templates/`.")
    manifest_path.write_text("\n".join(lines))
    print(f"\nWrote manifest: {manifest_path}")

    # Generate per-crate template files
    templates_dir = ROOT / "tools" / "dispatcher-gen" / "templates"
    templates_dir.mkdir(exist_ok=True)
    for crate, fns in sorted(by_crate.items()):
        crate_slug = crate.replace("/", "_")
        out = templates_dir / f"{crate_slug}_wrappers.rs"
        lines = [
            f"// AUTO-GENERATED by tools/dispatcher-gen/dispatcher-gen.py",
            f"// Source: {crate}/src/services.rs ({len(fns)} fns)",
            f"//",
            f"// Add to {crate}/src/services.rs and {crate}/src/lib.rs.",
            f"// Template: Wave 192 dispatch_admit_student pattern.",
            "",
            "use educore_dispatcher::CommandDispatcher;",
            "",
        ]
        for f in fns:
            lines.append(generate_command_bounds(f))
            lines.append(generate_wrapper(f))
            lines.append("")
        out.write_text("\n".join(lines))
        print(f"  Wrote {out.relative_to(ROOT)} ({len(fns)} fns)")

    return 0


if __name__ == "__main__":
    sys.exit(main())
