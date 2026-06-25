#!/usr/bin/env python3
"""Enumerate every spec aggregate, workflow, and capability as a roadmap item.

Reads the 11 domain spec folders and produces a TOML file containing one
`[[item]]` per spec'd aggregate / workflow / capability that lacks test
coverage. The script determines test coverage by looking for matching
test files in the corresponding domain crate's `tests/` directory.

Usage:
    python3 scripts/enumerate-spec-coverage.py [--output PATH]

Default output: `docs/audit_reports/remediation/12-roadmap-gaps-coverage.toml`
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
SPECS = REPO_ROOT / "docs" / "specs"
OUTPUT = REPO_ROOT / "docs" / "audit_reports" / "remediation" / "12-roadmap-gaps-coverage.toml"

# Mapping: domain (kebab-case spec dir) -> crate directory
DOMAIN_CRATE_MAP = {
    "library":       ("crates/domains/library", "educore-library"),
    "attendance":    ("crates/domains/attendance", "educore-attendance"),
    "communication": ("crates/domains/communication", "educore-communication"),
    "documents":     ("crates/domains/documents", "educore-documents"),
    "academic":      ("crates/domains/academic", "educore-academic"),
    "cms":           ("crates/domains/cms", "educore-cms"),
    "facilities":    ("crates/domains/facilities", "educore-facilities"),
    "assessment":    ("crates/domains/assessment", "educore-assessment"),
    "finance":       ("crates/domains/finance", "educore-finance"),
    "hr":            ("crates/domains/hr", "educore-hr"),
    "events":        ("crates/cross-cutting/events-domain", "educore-events-domain"),
}


def to_snake_case(name: str) -> str:
    """Convert AggregateName -> aggregate_name.

    Handles CamelCase, including consecutive capitals (e.g., NewsCategory -> news_category).
    """
    s1 = re.sub(r"([A-Z]+)([A-Z][a-z])", r"\1_\2", name)
    s2 = re.sub(r"([a-z0-9])([A-Z])", r"\1_\2", s1)
    return s2.lower()


def extract_aggregates(spec_file: Path) -> list[str]:
    """Extract root aggregate names from aggregates.md.

    A heading is a root aggregate if it has `### Commands` and `### Events` subsections.
    Dedupes (some specs list aggregates twice — main + appendix).
    """
    if not spec_file.exists():
        return []
    text = spec_file.read_text()
    aggregates: list[str] = []
    seen: set[str] = set()
    sections = re.split(r"^## ", text, flags=re.MULTILINE)
    for section in sections[1:]:
        heading = section.split("\n", 1)[0].strip()
        if not heading or heading.startswith("###"):
            continue
        has_commands = "### Commands" in section or "### commands" in section
        has_events = "### Events" in section or "### events" in section
        if has_commands and has_events and heading not in seen:
            aggregates.append(heading)
            seen.add(heading)
    return aggregates


def extract_workflows(spec_file: Path) -> list[str]:
    """Extract workflow names from workflows.md.

    Workflows are ## headings (deduped).
    """
    if not spec_file.exists():
        return []
    text = spec_file.read_text()
    workflows: list[str] = []
    seen: set[str] = set()
    for match in re.finditer(r"^## (.+)$", text, re.MULTILINE):
        heading = match.group(1).strip()
        if heading and not heading.startswith("###") and heading not in seen:
            workflows.append(heading)
            seen.add(heading)
    return workflows


def extract_capabilities(spec_file: Path) -> list[str]:
    """Extract capability names from permissions.md.

    Capabilities are typically ### headings or `Domain.Aggregate.Action` strings.
    """
    if not spec_file.exists():
        return []
    text = spec_file.read_text()
    capabilities: list[str] = []
    seen: set[str] = set()

    # Look for `Domain.Aggregate.Action` style strings
    for match in re.finditer(r"`?([A-Z][a-zA-Z]+\.[A-Z][a-zA-Z]+\.[A-Z][a-zA-Z]+)`?", text):
        cap = match.group(1)
        if cap not in seen:
            capabilities.append(cap)
            seen.add(cap)

    # Also look for ### Capability headings
    for match in re.finditer(r"^### ([A-Z][a-zA-Z]+)$", text, re.MULTILINE):
        cap = match.group(1).strip()
        if cap and cap not in seen:
            capabilities.append(cap)
            seen.add(cap)

    return capabilities


def has_test_file(crate_dir: Path, aggregate: str) -> bool:
    """Check if the aggregate has integration test coverage.

    Two cases count as covered:
    1. Dedicated file: `tests/<aggregate_snake>.rs` exists
    2. Generic file coverage: a function/module name in any test file
       matches the aggregate's snake_case prefix (e.g., `fn wallet_...`,
       `mod wallet_...`).
    """
    tests_dir = crate_dir / "tests"
    if not tests_dir.exists():
        return False

    snake = to_snake_case(aggregate)

    # 1. Dedicated file
    if (tests_dir / f"{snake}.rs").exists():
        return True

    # 2. Dedicated subdirectory
    if (tests_dir / snake).is_dir():
        return True

    # 3. Function/module naming: look for snake_case aggregate name as a
    #    prefix of test function or module names
    prefix = re.compile(rf"\b(?:fn|mod|async fn)\s+{re.escape(snake)}[_\b]")
    for test_file in tests_dir.glob("*.rs"):
        try:
            text = test_file.read_text()
        except OSError:
            continue
        if prefix.search(text):
            return True

    return False


def has_workflow_test(crate_dir: Path, workflow: str) -> bool:
    """Heuristic: does a workflow test exist?

    Checks if the workflow name appears in any existing test file.
    """
    tests_dir = crate_dir / "tests"
    if not tests_dir.exists():
        return False
    workflow_words = re.findall(r"[A-Za-z]+", workflow.lower())
    if not workflow_words:
        return False
    for test_file in tests_dir.glob("*.rs"):
        text = test_file.read_text().lower()
        if any(w in text for w in workflow_words if len(w) > 4):
            return True
    return False


def toml_escape(s: str) -> str:
    """Escape a string for use in TOML double-quoted string."""
    return s.replace("\\", "\\\\").replace('"', '\\"')


def emit_item(item_id: str, area: str, priority: str, description: str,
              source: str, check: str) -> str:
    return (
        f'[[item]]\n'
        f'id = "{item_id}"\n'
        f'priority = "{priority}"\n'
        f'area = "{area}"\n'
        f'description = "{toml_escape(description)}"\n'
        f'source = "{toml_escape(source)}"\n'
        f'check = "{toml_escape(check)}"\n'
    )


def build_coverage() -> dict[str, list[str]]:
    """For each domain, enumerate untested aggregates, workflows, capabilities."""
    result: dict[str, list[str]] = {"items": [], "summary": {}}

    for domain, (crate_rel, crate_name) in DOMAIN_CRATE_MAP.items():
        crate_dir = REPO_ROOT / crate_rel
        spec_dir = SPECS / domain

        # Enumerate aggregates
        aggregates = extract_aggregates(spec_dir / "aggregates.md")
        untested_aggs = []
        for agg in aggregates:
            if not has_test_file(crate_dir, agg):
                untested_aggs.append(agg)

        # Enumerate workflows
        workflows = extract_workflows(spec_dir / "workflows.md")
        untested_wfs = []
        for wf in workflows:
            # Skip section headings like "Idempotency" or "Audit Requirements"
            if wf in ("Idempotency", "Audit Requirements", "Failure paths",
                      "Pre-conditions", "Capabilities"):
                continue
            if not has_workflow_test(crate_dir, wf):
                untested_wfs.append(wf)

        # Enumerate capabilities (from permissions.md)
        capabilities = extract_capabilities(spec_dir / "permissions.md")
        untested_caps = []
        for cap in capabilities:
            # Capability checks are hard — skip the heuristic, just list all
            untested_caps.append(cap)

        result["summary"][domain] = {
            "total_aggregates": len(aggregates),
            "tested_aggregates": len(aggregates) - len(untested_aggs),
            "untested_aggregates": len(untested_aggs),
            "total_workflows": len(workflows),
            "untested_workflows": len(untested_wfs),
            "total_capabilities": len(capabilities),
            "untested_capabilities": len(untested_caps),
        }

        # Generate items
        domain_upper = domain.replace("-", "_").upper()
        for agg in untested_aggs:
            agg_snake = to_snake_case(agg)
            item_id = f"AGG-{domain_upper}-{agg_snake.upper()}"
            description = f"{domain}: {agg} aggregate has no integration test"
            source = f"docs/specs/{domain}/aggregates.md ## {agg}"
            check = f"file-exists:{crate_rel}/tests/{agg_snake}.rs"
            result["items"].append(
                emit_item(item_id, "TESTS", "P2", description, source, check)
            )

        for wf in untested_wfs:
            wf_snake = to_snake_case(wf).replace(" ", "_")
            item_id = f"WF-{domain_upper}-{wf_snake.upper()[:60]}"
            description = f"{domain}: '{wf}' workflow not implemented"
            source = f"docs/specs/{domain}/workflows.md ## {wf}"
            # Workflow check: look for function that mentions the workflow
            check = f"file:{crate_rel}/src/services.rs regex:{wf[:30]}"
            result["items"].append(
                emit_item(item_id, "WORKFLOWS", "P2", description, source, check)
            )

        for cap in untested_caps:
            cap_snake = to_snake_case(cap)
            item_id = f"CAP-{domain_upper}-{cap_snake.upper()[:60]}"
            description = f"{domain}: {cap} capability not wired in RBAC"
            source = f"docs/specs/{domain}/capabilities.md ## {cap}"
            check = f"file:crates/cross-cutting/rbac/src/value_objects.rs regex:{cap[:30]}"
            result["items"].append(
                emit_item(item_id, "RBAC", "P3", description, source, check)
            )

    return result


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=OUTPUT,
                        help="output TOML file")
    parser.add_argument("--verbose", "-v", action="store_true")
    args = parser.parse_args()

    result = build_coverage()

    # Build the TOML output
    lines = [
        "# Production Readiness Roadmap — exhaustive coverage gaps",
        "#",
        "# Auto-generated by `scripts/enumerate-spec-coverage.py` from 11 domain specs.",
        "# Do NOT edit by hand — regenerate.",
        "",
    ]

    # Header summary
    lines.append("# Coverage summary:")
    total_aggs = sum(s["total_aggregates"] for s in result["summary"].values())
    tested_aggs = sum(s["tested_aggregates"] for s in result["summary"].values())
    total_wfs = sum(s["total_workflows"] for s in result["summary"].values())
    total_caps = sum(s["total_capabilities"] for s in result["summary"].values())
    lines.append(f"#   Aggregates: {tested_aggs}/{total_aggs} tested ({100*tested_aggs//max(total_aggs,1)}%)")
    lines.append(f"#   Workflows:  {total_wfs} total in specs")
    lines.append(f"#   Capabilities: {total_caps} total in specs")
    lines.append(f"#   Items:      {len(result['items'])}")
    lines.append("")
    lines.append("# ============================================================================")
    lines.append("# Per-aggregate coverage (P2)")
    lines.append("# ============================================================================")
    lines.append("")

    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text("\n".join(lines) + "\n" + "\n".join(result["items"]))

    if args.verbose:
        print(f"Wrote {len(result['items'])} items to {args.output}")
        for domain, summary in result["summary"].items():
            print(f"  {domain}: {summary['tested_aggregates']}/{summary['total_aggregates']} aggs, "
                  f"{summary['untested_workflows']}/{summary['total_workflows']} wfs, "
                  f"{summary['untested_capabilities']} caps untested")

    return 0


if __name__ == "__main__":
    sys.exit(main())
