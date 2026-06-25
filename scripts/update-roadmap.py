#!/usr/bin/env python3
"""Update the production-readiness roadmap from the data file.

Reads `docs/audit_reports/remediation/12-roadmap-data.toml`, runs each
item's `check`, and regenerates the COMPUTED sections of
`docs/audit_reports/remediation/12-production-readiness-roadmap.md`.

Check types:
  cmd:<shell command>      exit 0 = done
  file:<path> regex:<pat>  grep matches = done
  file-exists:<path>       file exists = done
  commit:<regex>           git log has matching commit = done
  manual                   no auto-check (always in-progress)
  duplicate:<other-id>     same status as the referenced item

Usage:
    python3 scripts/update-roadmap.py [--dry-run] [--verbose]
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
import tomllib
from datetime import datetime, timezone
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
DATA_FILES = [
    REPO_ROOT / "docs" / "audit_reports" / "remediation" / "12-roadmap-data.toml",
    REPO_ROOT / "docs" / "audit_reports" / "remediation" / "12-roadmap-gaps-audit.toml",
]
MD_FILE = REPO_ROOT / "docs" / "audit_reports" / "remediation" / "12-production-readiness-roadmap.md"

# Section markers in the .md file
START_MARKER = "<!-- COMPUTED:"
END_MARKER = "<!-- END COMPUTED -->"


# ============================================================================
# Check execution
# ============================================================================

def run_check(check: str, verbose: bool = False) -> tuple[bool, str]:
    """Run a check and return (passed, evidence).

    `passed=True` means the item is DONE.

    Suffix `!` negates the check (DONE when the underlying predicate fails).
    E.g. `file:path regex:stub_text!` means DONE when the stub text is absent.
    """
    if not check or check == "manual":
        return False, "manual review"

    if check.startswith("manual:"):
        return False, f"manual: {check[7:]}"

    # Strip trailing `!` for negation
    negate = False
    body = check
    if body.endswith("!"):
        negate = True
        body = body[:-1]

    if body.startswith("cmd:"):
        done, evidence = _run_cmd_check(body[4:], verbose)
        return (not done if negate else done), evidence

    if body.startswith("file-exists:"):
        path = REPO_ROOT / body[len("file-exists:"):]
        exists = path.exists()
        done = (not exists if negate else exists)
        return done, f"{path.name} {'exists' if exists else 'missing'}"

    if body.startswith("file:"):
        # Format: file:<path> regex:<pattern>
        m = re.match(r"file:(\S+)\s+regex:(.+)", body, re.DOTALL)
        if not m:
            return False, f"malformed file check: {check}"
        path = REPO_ROOT / m.group(1)
        pattern = m.group(2)
        if not path.exists():
            return False, f"{path.name} missing"
        try:
            text = path.read_text()
            # Treat `regex:` containing alternation OR literal substring
            if "|" in pattern and not any(c in pattern for c in "[]()*+.?\\"):
                matches = any(opt in text for opt in pattern.split("|"))
            else:
                matches = bool(re.search(pattern, text))
            done = (not matches if negate else matches)
            return done, f"{path.name}:{pattern[:40]}"
        except OSError as e:
            return False, f"read error: {e}"

    if body.startswith("commit:"):
        pattern = body[len("commit:"):]
        try:
            out = subprocess.run(
                ["git", "log", "--oneline", f"--grep={pattern}"],
                cwd=REPO_ROOT,
                capture_output=True,
                text=True,
                timeout=10,
            )
            matches = bool(out.stdout.strip())
            done = (not matches if negate else matches)
            return done, f"git log grep: {pattern}"
        except (subprocess.TimeoutExpired, OSError) as e:
            return False, f"git error: {e}"

    return False, f"unknown check type: {check[:40]}"


def _run_cmd_check(cmd: str, verbose: bool) -> tuple[bool, str]:
    """Run a shell command, treat exit 0 as DONE.

    For long-running commands (cargo test, etc.) we use a short timeout
    and fall back to manual if it takes too long.
    """
    try:
        out = subprocess.run(
            cmd,
            shell=True,
            cwd=REPO_ROOT,
            capture_output=True,
            text=True,
            timeout=60,  # 60s timeout for any single check
        )
        return out.returncode == 0, f"exit {out.returncode}"
    except subprocess.TimeoutExpired:
        return False, "timeout (>60s) — likely needs manual run"
    except OSError as e:
        return False, f"exec error: {e}"


# ============================================================================
# Status computation
# ============================================================================

def evaluate_all(data: dict, verbose: bool = False) -> dict[str, tuple[bool, str]]:
    """Return a map of item_id -> (done, evidence)."""
    results: dict[str, tuple[bool, str]] = {}
    status_by_id: dict[str, str] = {}  # 'x' / '~' / ' '

    # First pass: evaluate non-duplicate items
    for item in data.get("item", []):
        item_id = item["id"]
        if item.get("check", "").startswith("duplicate:"):
            # Defer — resolve after first pass
            continue
        done, evidence = run_check(item.get("check", ""), verbose)
        results[item_id] = (done, evidence)

    # Second pass: resolve duplicates
    for item in data.get("item", []):
        item_id = item["id"]
        check = item.get("check", "")
        if check.startswith("duplicate:"):
            other_id = check[len("duplicate:"):]
            if other_id in results:
                results[item_id] = results[other_id]
            else:
                results[item_id] = (False, f"duplicate of unknown {other_id}")

    # Evaluate gates
    gate_results: dict[str, tuple[bool, str]] = {}
    for gate in data.get("gate", []):
        gate_id = gate["id"]
        done, evidence = run_check(gate.get("check", ""), verbose)
        gate_results[gate_id] = (done, evidence)

    return {"items": results, "gates": gate_results}


# ============================================================================
# Markdown rendering
# ============================================================================

def checkbox(done: bool, manual: bool = False) -> str:
    """Return a checkbox string. Manual items are always [~]."""
    if manual:
        return "[~]"
    return "[x]" if done else "[ ]"


def render_status(data: dict, item_results: dict, gate_results: dict) -> str:
    items = data.get("item", [])
    total = len(items)

    # Determine which items are manual (no auto-check) and not done
    manual_ids = {
        it["id"] for it in items
        if it.get("check", "").startswith("manual") or not it.get("check", "")
    }

    done = sum(1 for d, _ in item_results.values() if d)
    in_progress = sum(
        1 for it in items
        if it["id"] in manual_ids and not item_results[it["id"]][0]
    )
    open_count = total - done - in_progress

    now = datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M UTC")
    return (
        "| Metric | Value |\n"
        "|---|---|\n"
        f"| Total items | {total} |\n"
        f"| Done (`[x]`) | {done} |\n"
        f"| In-progress (`[~]`) | {in_progress} |\n"
        f"| Open (`[ ]`) | {open_count} |\n"
        f"| Last update | {now} |\n"
        f"| Last commit covered | `{data['meta'].get('main_head', '?')}` |\n"
    )


def render_gates(data: dict, gate_results: dict) -> str:
    lines = []
    for gate in data.get("gate", []):
        done, evidence = gate_results[gate["id"]]
        cb = checkbox(done)
        lines.append(f"- {cb} **{gate['name']}:** {gate['description']}")
        lines.append(f"      _check: `{gate.get('check', '')}` → {evidence}_")
    return "\n".join(lines) + "\n"


def render_items(data: dict, item_results: dict, section: str) -> str:
    """Render items for one COMPUTED section.

    section format: items.<priority>.<area>  OR  items.<priority>
    """
    parts = section.split(".")
    if len(parts) < 2 or parts[0] != "items":
        return ""

    priority = parts[1].upper()  # e.g., "P0"
    area = parts[2].upper() if len(parts) > 2 else None

    lines = []
    matched = 0
    for item in data.get("item", []):
        if item["priority"].upper() != priority:
            continue
        if area is not None and item.get("area", "").upper() != area:
            continue
        matched += 1
        item_id = item["id"]
        done, evidence = item_results.get(item_id, (False, "?"))
        check = item.get("check", "")
        is_manual = check.startswith("manual") or not check
        cb = checkbox(done, is_manual)
        lines.append(f"- {cb} **{item_id}** {item['description']}")
        lines.append(f"      **Source:** {item.get('source', '?')}")
        if check:
            lines.append(f"      **Check:** `{check[:80]}{'...' if len(check) > 80 else ''}` → _{evidence}_")
        else:
            lines.append(f"      **Check:** _(no check — manual)_")
        lines.append("")

    if matched == 0:
        return "_no items in this section_\n"

    return "\n".join(lines)


def render_section(data: dict, item_results: dict, gate_results: dict, section: str) -> str:
    """Dispatch to the right renderer based on section name."""
    if section == "status":
        return render_status(data, item_results, gate_results)
    if section == "gates":
        return render_gates(data, gate_results)
    if section.startswith("items."):
        return render_items(data, item_results, section)
    return f"_unknown section: {section}_\n"


# ============================================================================
# Markdown patching
# ============================================================================

def update_md(data: dict, item_results: dict, gate_results: dict, dry_run: bool, verbose: bool) -> bool:
    md = MD_FILE.read_text()
    original_md = md

    # Find all COMPUTED blocks: <!-- COMPUTED:name --> ... <!-- END COMPUTED -->
    pattern = re.compile(
        rf"{re.escape(START_MARKER)}([^\s>]+)\s*-->\s*\n(.*?){re.escape(END_MARKER)}",
        re.DOTALL,
    )

    def replace_block(match: re.Match) -> str:
        section = match.group(1).strip()
        rendered = render_section(data, item_results, gate_results, section)
        if verbose:
            print(f"  [{section}] rendered {len(rendered)} chars")
        return f"{START_MARKER}{section} -->\n{rendered.rstrip()}\n{END_MARKER}"

    new_md = pattern.sub(replace_block, md)

    if new_md == original_md:
        print("No changes needed — sections already match")
        return False

    if dry_run:
        print("DRY RUN — would update sections:")
        # Show first 200 chars of each change
        for m in pattern.finditer(original_md):
            section = m.group(1).strip()
            new_content = render_section(data, item_results, gate_results, section)
            print(f"  [{section}] {len(new_content)} chars")
        return True

    MD_FILE.write_text(new_md)
    print(f"Updated {MD_FILE.relative_to(REPO_ROOT)}")
    return True


# ============================================================================
# Main
# ============================================================================

def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--dry-run", action="store_true", help="don't write; just show what would change")
    parser.add_argument("--verbose", "-v", action="store_true", help="print per-check details")
    args = parser.parse_args()

    if not MD_FILE.exists():
        print(f"ERROR: roadmap .md not found: {MD_FILE}", file=sys.stderr)
        return 1

    # Load all data files and merge
    data: dict = {"item": [], "gate": [], "meta": {}}
    for path in DATA_FILES:
        if not path.exists():
            print(f"ERROR: data file not found: {path}", file=sys.stderr)
            return 1
        with path.open("rb") as f:
            chunk = tomllib.load(f)
        for key in ("item", "gate"):
            data[key].extend(chunk.get(key, []))
        # meta is from the primary file only
        if path == DATA_FILES[0]:
            data["meta"] = chunk.get("meta", {})

    if args.verbose:
        print(f"Loaded {len(data['item'])} items, {len(data['gate'])} gates from {len(DATA_FILES)} files")

    if args.verbose:
        print("Running checks:")
    results = evaluate_all(data, args.verbose)
    if args.verbose:
        for item_id, (done, evidence) in sorted(results["items"].items()):
            print(f"  {item_id}: {'DONE' if done else 'open'} ({evidence})")
        for gate_id, (done, evidence) in sorted(results["gates"].items()):
            print(f"  Gate-{gate_id}: {'DONE' if done else 'open'} ({evidence})")

    changed = update_md(data, results["items"], results["gates"], args.dry_run, args.verbose)
    return 0 if changed else 0  # both "changed" and "no changes" are success


if __name__ == "__main__":
    sys.exit(main())
