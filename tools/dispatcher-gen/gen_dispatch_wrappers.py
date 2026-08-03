#!/usr/bin/env python3
"""
gen_dispatch_wrappers — generate dispatch_X wrappers for service functions.

For each crate, scans src/services.rs for `pub fn X<C, G>(cmd: XCommand, ...)`
signatures and produces corresponding `pub async fn dispatch_X<C, G>(...)`
wrappers that call `CommandDispatcher::dispatch`.

Capability strings derive from the command name (verb.target.domain).
Cloning the command inside the closure satisfies the borrow checker
(dispatch borrows &cmd while the closure moves cmd).

Usage:
    python3 tools/dispatcher-gen/gen_dispatch_wrappers.py --crate hr
    python3 tools/dispatcher-gen/gen_dispatch_wrappers.py --crate academic --crate library
    python3 tools/dispatcher-gen/gen_dispatch_wrappers.py --all
"""
from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def detect_domain(crate_path: str) -> str:
    """crates/domains/hr -> 'hr'; cross-cutting/platform -> 'platform'."""
    parts = Path(crate_path).parts
    if "domains" in parts:
        return parts[parts.index("domains") + 1]
    if "cross-cutting" in parts:
        return parts[parts.index("cross-cutting") + 1]
    if "adapters" in parts:
        return parts[parts.index("adapters") + 1]
    return "unknown"


def derive_action_target(cmd: str) -> tuple[str, str]:
    """AdmitStudentCommand -> ('admit', 'student')."""
    if not cmd.endswith("Command"):
        return ("unknown", "unknown")
    name = cmd[:-len("Command")]
    action = "unknown"
    for verb in ("Admit", "Update", "Create", "Delete", "Set", "Assign",
                 "Register", "Retire", "Mark", "Unlink", "Link", "Promote",
                 "Graduate", "Suspend", "Reinstate", "Withdraw", "Transfer",
                 "Close", "Copy", "Reassign", "Unassign", "Swap", "Cancel",
                 "Open", "Renew", "Bind", "Approve", "Reject", "Hire",
                 "Request", "Run", "Record", "Refresh"):
        if name.startswith(verb):
            action = verb.lower()
            name = name[len(verb):]
            break
    target = name
    if target.endswith("ies"):
        target = target[:-3] + "y"
    elif target.endswith("es") and not target.endswith("ss"):
        target = target[:-2]
    elif target.endswith("s") and not target.endswith("ss"):
        target = target[:-1]
    target = target[:1].lower() + target[1:] if target else "unknown"
    return (action, target)


def parse_service_fns(services_rs: Path, commands_with_bounds: set[str]) -> list[dict]:
    """Scan services.rs for service fn signatures.
    Only returns fns whose:
    - First arg is a Command
    - Command has a CommandBounds impl
    - No port traits beyond UniquenessChecker (transaction is OK)
    """
    src = services_rs.read_text()
    out = []
    # Match: pub (async)? fn NAME<C, G>( args ) -> RET
    # Stop the return-type capture at `where` clause or `{`.
    for m in re.finditer(
        r"^pub\s+(?:async\s+)?fn\s+(\w+)\s*<C,\s*G>\s*\(([^)]*)\)\s*(?:->\s*([^{]+?))?(?=\s+where|\s*\{)",
        src, re.MULTILINE | re.DOTALL,
    ):
        name = m.group(1)
        args = m.group(2).strip()
        ret = (m.group(3) or "()").strip()
        if name.startswith("dispatch_"):
            continue
        first = args.split(",")[0].strip()
        if not first.endswith("Command"):
            continue
        cmd = first.split()[-1]
        # Skip commands without CommandBounds impl
        if cmd not in commands_with_bounds:
            continue
        # Detect UniquenessChecker and capture the actual trait name
        has_u = False
        uniqueness_trait = "UniquenessChecker"
        u_match = re.search(r"uniqueness:\s*&dyn\s+(\w+)", args)
        if u_match:
            has_u = True
            uniqueness_trait = u_match.group(1)
        # Detect other &dyn ports (not UniquenessChecker, not Transaction)
        other_ports = re.findall(r"&dyn\s+(\w+)", args)
        other_ports = [p for p in other_ports if "UniquenessChecker" not in p]
        if other_ports:
            # Skip fns that take port traits we can't easily wire
            continue
        # Strip outer Result<...>
        bare_ret = ret.strip()
        m_re = re.match(r"^Result\s*<(.+)>$", bare_ret, re.DOTALL)
        if m_re:
            inner = m_re.group(1).strip()
            depth = 0
            for i, ch in enumerate(inner):
                if ch == "<":
                    depth += 1
                elif ch == ">":
                    depth -= 1
                    if depth == 0:
                        bare_ret = inner[:i + 1]
                        break
        out.append({
            "name": name,
            "cmd": cmd,
            "return": bare_ret,
            "has_uniqueness": has_u,
            "uniqueness_trait": uniqueness_trait,
            "other_ports": other_ports,
        })
    return out


def gen_wrapper(fn: dict, domain: str) -> str:
    """Generate the dispatch_X wrapper function."""
    action, target = derive_action_target(fn["cmd"])
    cap = f"{domain}.{target}.{action}"
    sig_args = [
        "    dispatcher: &CommandDispatcher,",
        f"    cmd: {fn['cmd']},",
        "    clock: &C,",
        "    ids: &G,",
    ]
    if fn["has_uniqueness"]:
        sig_args.append(f"    uniqueness: &dyn {fn['uniqueness_trait']},")
    pass_args = ["cmd.clone()", "clock", "ids"]
    if fn["has_uniqueness"]:
        pass_args.append("uniqueness")
    body = ",\n            ".join(pass_args)
    return f"""/// Dispatcher wrapper for [`{fn['name']}].
///
/// Mirrors the Wave 192 + Wave 206 pattern: clones `cmd`
/// inside the closure to satisfy the borrow checker, then
/// runs the full `CommandDispatcher::dispatch` pipeline
/// (RBAC → txn → idempotency → service → outbox → audit →
/// idempotency record → bus publish).
pub async fn dispatch_{fn['name']}<C, G>(
{sig_args[0]}
{sig_args[1]}
{sig_args[2]}
{sig_args[3]}
{chr(10).join(sig_args[4:])}) -> Result<{fn['return']}>
where
    C: Clock + ?Sized + Send + Sync,
    G: IdGenerator + ?Sized + Send + Sync,
{{
    use educore_dispatcher::CommandBounds as _;
    dispatcher
        .dispatch(&cmd, &["{cap}"], || async {{
            {fn['name']}::<C, G>({body})
        }})
        .await
}}
"""


def find_commands_with_bounds(crate_dir: Path) -> set[str]:
    """Find all command types that have an `impl CommandBounds` block in
    either commands.rs or services.rs."""
    names: set[str] = set()
    for sub in ("commands.rs", "services.rs"):
        rs = crate_dir / "src" / sub
        if rs.exists():
            src = rs.read_text()
            for m in re.finditer(
                r"impl\s+(?:educore_dispatcher::)?CommandBounds\s+for\s+(\w+)\s*\{",
                src,
            ):
                names.add(m.group(1))
    return names


def wire_crate(crate_dir: Path, dry_run: bool = False) -> dict:
    """Wire dispatch_X wrappers for a crate's service fns.
    Returns stats dict."""
    services_rs = crate_dir / "src" / "services.rs"
    if not services_rs.exists():
        return {"crate": str(crate_dir), "status": "skip:no-services", "added": 0}
    commands_with_bounds = find_commands_with_bounds(crate_dir)
    fns = parse_service_fns(services_rs, commands_with_bounds)
    if not fns:
        return {
            "crate": str(crate_dir),
            "status": "skip:no-eligible-fns",
            "added": 0,
        }
    domain = detect_domain(str(crate_dir.relative_to(ROOT)))
    # Skip if dispatch_ already present (idempotency)
    existing = services_rs.read_text()
    already_wrapped = sum(1 for f in fns if f"pub async fn dispatch_{f['name']}" in existing)
    to_add = [f for f in fns if f"pub async fn dispatch_{f['name']}" not in existing]
    if not to_add:
        return {
            "crate": str(crate_dir),
            "status": "skip:already-wired",
            "added": 0,
            "total": len(fns),
        }
    wrappers = "\n".join(gen_wrapper(f, domain) for f in to_add)
    if dry_run:
        return {
            "crate": str(crate_dir),
            "status": "dry-run",
            "would_add": len(to_add),
            "total": len(fns),
        }
    # Append before #[cfg(test)] if present
    text = existing
    block = (
        f"\n// =============================================================================\n"
        f"// Auto-generated dispatch_X wrappers (Wave 208+)\n"
        f"// Pattern: Wave 192 dispatch_admit_student + Wave 206 dispatch_hire_staff\n"
        f"// Tools: tools/dispatcher-gen/gen_dispatch_wrappers.py\n"
        f"// =============================================================================\n\n"
        f"{wrappers}"
    )
    if "#[cfg(test)]" in text:
        parts = text.split("#[cfg(test)]", 1)
        new_text = parts[0].rstrip() + "\n\n" + block + "\n\n#[cfg(test)]" + parts[1]
    else:
        new_text = text.rstrip() + "\n\n" + block
    services_rs.write_text(new_text)
    return {
        "crate": str(crate_dir),
        "status": "written",
        "added": len(to_add),
        "total": len(fns),
    }


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--crate", action="append", help="Crate path(s), e.g. 'domains/hr'")
    ap.add_argument("--all", action="store_true", help="Process all crates")
    ap.add_argument("--dry-run", action="store_true", help="Just show what would be done")
    ap.add_argument("--verify", action="store_true", help="Run cargo build for each wired crate")
    args = ap.parse_args()

    if not args.crate and not args.all:
        ap.error("Specify --crate or --all")

    crates = []
    for cargo_toml in ROOT.glob("crates/**/Cargo.toml"):
        if "infra/core" in str(cargo_toml) or "infra/storage" in str(cargo_toml) \
           or "infra/query-derive" in str(cargo_toml) or "tools" in str(cargo_toml) \
           or "adapters" in str(cargo_toml):
            continue
        crates.append(cargo_toml.parent)

    if args.crate:
        crates = [c for c in crates if any(arg in str(c) for arg in args.crate)]

    print(f"Processing {len(crates)} crates")
    results = []
    for c in sorted(crates):
        result = wire_crate(c, dry_run=args.dry_run)
        results.append(result)
        rel = result["crate"].replace(str(ROOT) + "/", "")
        if result["status"] == "written":
            print(f"  + {rel}: added {result['added']} wrappers")
        elif result["status"] == "dry-run":
            print(f"  ? {rel}: would add {result['would_add']}/{result['total']} wrappers")
        else:
            print(f"  - {rel}: {result['status']}")

    if args.verify and not args.dry_run:
        print("\nVerifying builds...")
        for r in results:
            if r["status"] == "written":
                pkg_match = re.search(r'name\s*=\s*"([^"]+)"', (Path(r["crate"]) / "Cargo.toml").read_text())
                if pkg_match:
                    pkg = pkg_match.group(1)
                    res = subprocess.run(
                        ["cargo", "build", "-p", pkg],
                        cwd=ROOT, capture_output=True, text=True, timeout=180,
                    )
                    flag = "✓" if res.returncode == 0 else "✗"
                    print(f"  {flag} {pkg}")
                    if res.returncode != 0:
                        # Print last few lines of stderr for context
                        for line in res.stderr.split("\n")[-10:]:
                            print(f"      {line}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
