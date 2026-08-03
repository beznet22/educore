#!/usr/bin/env python3
"""
add_tenant_fields — add `tenant: TenantContext` field to stub commands.

Strategy: for each `pub struct XCommand { pub id: ..., pub school_id: ... }`
that lacks a tenant field, add `pub tenant: TenantContext,` as the FIRST
field. This makes the command dispatcher-eligible without changing the
underlying aggregate.

Run after: dispatcher-gen generates CommandBounds impls.
Run before: gen_dispatch_wrappers generates dispatch_X wrappers.
"""
from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def scan_stub_commands(rs_path: Path) -> list[str]:
    """Find all XCommand structs without a tenant: TenantContext field."""
    if not rs_path.exists():
        return []
    src = rs_path.read_text()
    out = []
    for m in re.finditer(r"pub\s+struct\s+(\w+Command)\s*\{([^}]*)\}", src, re.DOTALL):
        name = m.group(1)
        body = m.group(2)
        if "tenant" in body and "TenantContext" in body:
            continue  # Already has tenant
        # Skip if the body is empty (true placeholder)
        if not body.strip():
            continue
        # Only add to commands that look like dispatchable commands
        # (have at least an id or school_id field)
        if "school_id" in body or "SchoolId" in body or "id:" in body:
            out.append(name)
    return out


def add_tenant_field(rs_path: Path, cmd_name: str) -> bool:
    """Add `pub tenant: TenantContext,` as first field of struct."""
    src = rs_path.read_text()
    pattern = re.compile(
        r"(pub\s+struct\s+" + re.escape(cmd_name) + r"\s*\{\s*\n)"
        r"(\s*///[^\n]*\n)*"  # skip doc comments
        r"(\s*//[^\n]*\n)*"   # skip line comments
        r"(\s*#\[[^\]]*\]\s*\n)*"  # skip attributes
        r"(\s*pub\s+id:\s*[^,\n]+,?\s*\n)",
    )
    new_src, n = pattern.subn(
        lambda m: m.group(1) + "    pub tenant: TenantContext,\n"
        + (m.group(5) if m.group(5) else ""),
        src,
        count=1,
    )
    if n > 0:
        rs_path.write_text(new_src)
        return True
    # Simpler fallback: just insert after the opening brace
    pattern2 = re.compile(
        r"(pub\s+struct\s+" + re.escape(cmd_name) + r"\s*\{\s*\n)",
    )
    new_src, n = pattern2.subn(
        lambda m: m.group(1) + "    pub tenant: TenantContext,\n",
        src,
        count=1,
    )
    if n > 0:
        rs_path.write_text(new_src)
        return True
    return False


def main():
    modified_total = 0
    crates_processed = 0
    for cargo_toml in ROOT.glob("crates/**/Cargo.toml"):
        if any(x in str(cargo_toml) for x in ["infra/", "tools/"]):
            continue
        crate_dir = cargo_toml.parent
        commands_rs = crate_dir / "src" / "commands.rs"
        services_rs = crate_dir / "src" / "services.rs"
        for rs in [commands_rs, services_rs]:
            if not rs.exists():
                continue
            stubs = scan_stub_commands(rs)
            if not stubs:
                continue
            for cmd in stubs:
                if add_tenant_field(rs, cmd):
                    modified_total += 1
            crates_processed += 1
    print(f"Added tenant field to {modified_total} commands across {crates_processed} files")
    return 0


if __name__ == "__main__":
    sys.exit(main())
