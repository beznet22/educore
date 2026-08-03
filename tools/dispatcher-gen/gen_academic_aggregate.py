#!/usr/bin/env python3
"""
gen_academic_aggregate — upgrade academic_aggregate_stub! macros.

Academic uses: academic_aggregate_stub! { /// doc
    pub struct X { id: XId } }
"""
from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def expand_academic_stub(crate_dir: Path, struct_name: str) -> bool:
    agg_rs = crate_dir / "src" / "aggregate.rs"
    if not agg_rs.exists():
        return False
    src = agg_rs.read_text()
    pattern = re.compile(
        r"academic_aggregate_stub!\s*\{\s*"
        r"(?:///[^\n]*\n\s*)*"
        r"pub struct\s+" + re.escape(struct_name) + r"\s*\{\s*id:\s*\w+\s*\}\s*"
        r"\}",
        re.DOTALL,
    )
    doc_match = pattern.search(src)
    if not doc_match:
        return False
    doc_block = doc_match.group(0)
    doc_comment_match = re.search(r"(///[^\n]*\n\s*)+", doc_block)
    doc_comment = doc_comment_match.group(0) if doc_comment_match else ""

    replacement = f"""/// Real aggregate (Wave 220 upgrade from academic_aggregate_stub!).
{doc_comment}pub struct {struct_name} {{
    /// The typed id.
    pub id: {struct_name}Id,
    /// The owning school (tenant anchor).
    pub school_id: SchoolId,
    /// Active status (Active | Retired).
    pub active_status: ActiveStatus,
    /// Monotonic version for optimistic concurrency.
    pub version: Version,
    /// Entity tag for change detection.
    pub etag: Etag,
    /// When the aggregate was first created.
    pub created_at: Timestamp,
    /// When the aggregate was last mutated.
    pub updated_at: Timestamp,
    /// User who created the aggregate.
    pub created_by: UserId,
    /// User who last mutated the aggregate.
    pub updated_by: UserId,
    /// Correlation id for trace propagation.
    pub correlation_id: CorrelationId,
    /// Last event id emitted.
    pub last_event_id: Option<EventId>,
}}

impl {struct_name} {{
    /// Returns `true` if the aggregate is currently active.
    #[must_use]
    pub fn is_active(&self) -> bool {{
        self.active_status.is_active()
    }}

    /// Soft-deletes the aggregate (sets `active_status = Retired`).
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
    crate_dir = ROOT / "crates" / "domains" / "academic"
    for s in args.structs:
        if expand_academic_stub(crate_dir, s):
            print(f"  + {s}")
        else:
            print(f"  ! {s} (no match)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
