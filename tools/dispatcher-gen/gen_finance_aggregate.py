#!/usr/bin/env python3
"""
gen_finance_aggregate — upgrade finance_aggregate_stub! stubs to real
aggregates with audit footer fields.

The finance domain uses a custom `finance_aggregate_stub!` macro that
emits `pub struct X { _id: () }` stubs. This tool replaces each stub
block with a real struct + audit footer + is_active + retire methods.

Usage:
    python3 tools/dispatcher-gen/gen_finance_aggregate.py --structs FeesGroup FeesType
"""
from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def expand_finance_stub(crate_dir: Path, struct_name: str) -> bool:
    """Replace `finance_aggregate_stub! { ... pub struct X { _id: () } }`
    with a real struct + audit footer."""
    agg_rs = crate_dir / "src" / "aggregate.rs"
    if not agg_rs.exists():
        return False
    src = agg_rs.read_text()
    # Pattern: finance_aggregate_stub! { <doc comments> pub struct X { _id: () } }
    pattern = re.compile(
        r"finance_aggregate_stub!\s*\{\s*"
        r"(?:///[^\n]*\n\s*)*"
        r"pub struct\s+" + re.escape(struct_name) + r"\s*\{\s*_id:\s*\(\)\s*\}\s*"
        r"\}",
        re.DOTALL,
    )
    # Get the doc comment for this struct
    doc_match = pattern.search(src)
    if not doc_match:
        return False
    doc_block = doc_match.group(0)
    doc_comment_match = re.search(r"(///[^\n]*\n\s*)+", doc_block)
    doc_comment = doc_comment_match.group(0) if doc_comment_match else ""

    replacement = f"""/// Real aggregate (Wave 218 upgrade from finance_aggregate_stub!).
{doc_comment}pub struct {struct_name} {{
    pub id: crate::value_objects::{struct_name}Id,
    pub school_id: SchoolId,
    pub active_status: ActiveStatus,
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
    #[must_use]
    pub fn is_active(&self) -> bool {{
        self.active_status.is_active()
    }}

    pub fn retire(&mut self, at: Timestamp, actor: UserId) {{
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
    }}
}}"""
    new_src, n = pattern.subn(replacement, src, count=1)
    if n == 0:
        return False
    agg_rs.write_text(new_src)
    return True


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--structs", nargs="+", required=True)
    args = ap.parse_args()

    crate_dir = ROOT / "crates" / "domains" / "finance"
    for s in args.structs:
        if expand_finance_stub(crate_dir, s):
            print(f"  + {s}")
        else:
            print(f"  ! {s} (no match)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
