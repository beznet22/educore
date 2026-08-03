#!/usr/bin/env python3
"""
gen_aggregate — upgrade a stub aggregate (id+school_id only) to a
real aggregate with audit footer fields.

Generates the struct fields + `is_active()` method + `retire()` method.
Per-aggregate invariants (validation, FSM transitions) are added by
humans; this tool produces only the mechanical boilerplate.
"""
from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def expand_stub(crate_dir: Path, struct_name: str, audit_footer: bool = True) -> bool:
    """Replace `pub struct X { pub id: XId, pub school_id: SchoolId }`
    with the full struct + audit footer."""
    agg_rs = crate_dir / "src" / "aggregate.rs"
    if not agg_rs.exists():
        return False
    src = agg_rs.read_text()
    pattern = re.compile(
        r"pub struct " + re.escape(struct_name) + r"\s*\{\s*"
        r"pub id:\s*[\w:]+(?:Id)?,\s*"
        r"pub school_id:\s*SchoolId,\s*"
        r"\}",
        re.DOTALL,
    )
    if audit_footer:
        replacement = f"""pub struct {struct_name} {{
    pub id: crate::value_objects::{struct_name}Id,
    pub school_id: SchoolId,
    /// Active status (Active | Retired).
    pub active_status: ActiveStatus,
    /// Audit footer (per `AGENTS.md`).
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub correlation_id: CorrelationId,
    pub last_event_id: Option<EventId>,
}}

impl {struct_name} {{
    /// Returns true if currently active.
    #[must_use]
    pub fn is_active(&self) -> bool {{
        self.active_status.is_active()
    }}

    /// Soft-deletes the aggregate.
    pub fn retire(&mut self, at: Timestamp, actor: UserId) {{
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
    }}
}}"""
    else:
        replacement = f"""pub struct {struct_name} {{
    pub id: crate::value_objects::{struct_name}Id,
    pub school_id: SchoolId,
    pub active_status: ActiveStatus,
}}"""
    new_src, n = pattern.subn(replacement, src)
    if n == 0:
        return False
    agg_rs.write_text(new_src)
    return True


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--crate", required=True, help="Crate directory name")
    ap.add_argument("--structs", nargs="+", required=True, help="Struct names to expand")
    args = ap.parse_args()

    crate_dir = ROOT / "crates" / args.crate
    if not crate_dir.exists():
        print(f"crate not found: {crate_dir}")
        return 1
    for struct_name in args.structs:
        if expand_stub(crate_dir, struct_name):
            print(f"  + {struct_name}")
        else:
            print(f"  ! {struct_name} (no match)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
