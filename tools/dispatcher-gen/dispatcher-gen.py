#!/usr/bin/env python3
"""
dispatcher-gen — generate `CommandBounds` impl blocks for service commands.

Scans all crates' `src/commands.rs` files, extracts the
`pub struct XCommand` definitions, and generates the
`impl educore_dispatcher::CommandBounds for XCommand { ... }` block.

The `CommandBounds` impl is the mechanical, boilerplate part of the
dispatcher wrapper. The wrapper function itself
(`dispatch_X`) needs per-fn customization (capability string,
extra port args, return-type handling), so it's not auto-generated.

Wave 192 established the pattern: `admit_student` + `AdmitStudentCommand`.
This tool scales the `CommandBounds` impl part across all 382 service
commands in 8 crates.

Usage:
    python3 tools/dispatcher-gen/dispatcher-gen.py --domain academic
    python3 tools/dispatcher-gen/dispatcher-gen.py --all
    python3 tools/dispatcher-gen/dispatcher-gen.py --manifest
"""
from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path
from typing import NamedTuple


ROOT = Path(__file__).resolve().parents[2]


class Command(NamedTuple):
    crate: str
    name: str           # e.g. "AdmitStudentCommand"
    snake: str          # e.g. "admit_student_command"
    has_tenant: bool    # True if struct has a `tenant: TenantContext` field


# Verbs that map to action strings (lowercase first letter).
# Domain-prefix mapping for capability inference (best-effort).
DOMAIN_PREFIX = {
    "academic": "academic",
    "hr": "hr",
    "finance": "finance",
    "assessment": "assessment",
    "attendance": "attendance",
    "facilities": "facilities",
    "library": "library",
    "platform": "platform",
}


def scan_commands(crate_dir: Path) -> list[Command]:
    """Scan a crate's src/commands.rs for pub struct XCommand."""
    commands = crate_dir / "src" / "commands.rs"
    if not commands.exists():
        return []
    src = commands.read_text()
    crate = "/".join(crate_dir.relative_to(ROOT).parts[1:3])
    out = []
    # Match: pub struct XCommand { ... }  — non-greedy, no nested braces
    for m in re.finditer(r"pub\s+struct\s+(\w+Command)\s*\{([^}]*)\}", src, re.DOTALL):
        name = m.group(1)
        body = m.group(2)
        # Convert CamelCase to snake_case
        snake = re.sub(r"(?<!^)(?=[A-Z])", "_", name).lower()
        has_tenant = "tenant" in body and "TenantContext" in body
        out.append(Command(crate=crate, name=name, snake=snake, has_tenant=has_tenant))
    return out


def derive_action_target(cmd_name: str) -> tuple[str, str]:
    """Derive (action, target) from command name.

    AdmitStudentCommand -> ('admit', 'student')
    CreateClassCommand -> ('create', 'class')
    HireStaffCommand -> ('hire', 'staff')
    """
    if not cmd_name.endswith("Command"):
        return ("unknown", "unknown")
    name = cmd_name[:-len("Command")]
    action = "unknown"
    for verb in ("Admit", "Update", "Create", "Delete", "Set", "Assign",
                 "Register", "Retire", "Mark", "Unlink", "Link", "Promote",
                 "Graduate", "Suspend", "Reinstate", "Withdraw", "Transfer",
                 "Close", "Copy", "Reassign", "Unassign", "Swap", "Cancel",
                 "Open", "Renew", "Bind", "Approve", "Reject", "Hire",
                 "Request", "Run"):
        if name.startswith(verb):
            action = verb.lower()
            name = name[len(verb):]
            break
    # Singularize trailing 's'
    target = name
    if target.endswith("ies"):
        target = target[:-3] + "y"
    elif target.endswith("es") and not target.endswith("ss"):
        target = target[:-2]
    elif target.endswith("s") and not target.endswith("ss"):
        target = target[:-1]
    target = target[:1].lower() + target[1:] if target else "unknown"
    return (action, target)


def derive_capability(crate: str, action: str, target: str) -> str:
    """Infer capability string like 'academic.student.create'."""
    domain = "unknown"
    for k, v in DOMAIN_PREFIX.items():
        if k in crate:
            domain = v
            break
    return f"{domain}.{target}.{action}"


def generate_bounds(cmd: Command) -> str:
    """Generate the CommandBounds impl block for the command."""
    action, target = derive_action_target(cmd.name)
    capability = derive_capability(cmd.crate, action, target)
    if not cmd.has_tenant:
        return (
            f"// SKIP {cmd.name}: no `tenant: TenantContext` field "
            f"(CommandBounds requires one)\n"
        )
    return (
        f"impl educore_dispatcher::CommandBounds for {cmd.name} {{\n"
        f"    fn tenant(&self) -> &TenantContext {{ &self.tenant }}\n"
        f"    fn command_type(&self) -> &'static str {{ \"{capability}\" }}\n"
        f"    fn idempotency_key(&self) -> Option<IdempotencyKey> {{ None }}\n"
        f"    fn action(&self) -> &'static str {{ \"{action}\" }}\n"
        f"    fn target_type(&self) -> &'static str {{ \"{target}\" }}\n"
        f"}}\n"
    )


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--all", action="store_true", help="Scan all crates")
    ap.add_argument("--domain", help="Scan a specific crate, e.g. 'academic'")
    ap.add_argument("--manifest", action="store_true",
                    help="Only print the manifest, do not generate files")
    args = ap.parse_args()

    all_cmds: list[Command] = []
    for cargo_toml in ROOT.glob("crates/**/Cargo.toml"):
        cmds = scan_commands(cargo_toml.parent)
        if cmds:
            all_cmds.extend(cmds)

    if args.domain:
        all_cmds = [c for c in all_cmds if args.domain in c.crate]

    by_crate: dict[str, list[Command]] = {}
    for c in all_cmds:
        by_crate.setdefault(c.crate, []).append(c)

    if args.manifest:
        print(f"=== {len(by_crate)} crates, {len(all_cmds)} commands ===\n")
        for crate, cmds in sorted(by_crate.items(), key=lambda x: -len(x[1])):
            ready = sum(1 for c in cmds if c.has_tenant)
            print(f"  {crate}: {len(cmds)} cmds ({ready} with tenant)")
        return 0

    templates_dir = ROOT / "tools" / "dispatcher-gen" / "templates"
    templates_dir.mkdir(exist_ok=True)
    print(f"Generating CommandBounds impls for {len(all_cmds)} commands...")
    for crate, cmds in sorted(by_crate.items()):
        crate_slug = crate.replace("/", "_")
        out = templates_dir / f"{crate_slug}_bounds.rs"
        lines = [
            f"// AUTO-GENERATED by tools/dispatcher-gen/dispatcher-gen.py",
            f"// Source: {crate}/src/commands.rs ({len(cmds)} commands)",
            f"//",
            f"// Each impl block derives action/target from the command name.",
            f"// Capability string is inferred from the crate path.",
            f"// Skip commands without a `tenant: TenantContext` field.",
            f"//",
            f"// Wire into {crate}/src/commands.rs (append at end).",
            f"// Template: Wave 192 AdmitStudentCommand impl.",
            "",
        ]
        generated = 0
        skipped = 0
        for c in cmds:
            block = generate_bounds(c)
            if block.startswith("// SKIP"):
                skipped += 1
            else:
                generated += 1
            lines.append(block)
        lines.append(
            f"// Generated: {generated}, Skipped (no tenant field): {skipped}"
        )
        out.write_text("\n".join(lines))
        print(f"  {crate}: {generated} generated, {skipped} skipped -> "
              f"{out.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
