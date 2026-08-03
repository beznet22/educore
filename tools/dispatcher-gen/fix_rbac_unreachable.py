#!/usr/bin/env python3
"""
Fix unreachable_patterns in crates/cross-cutting/rbac/src/value_objects.rs.

Strategy: For each match block, scan arms in order. Keep only the FIRST
occurrence of each (variant, label) pair. Remove any subsequent arm
that maps the same (variant, label). For wildcard arms, strip variants
already covered by earlier arms with a different label.

This ensures:
1. No duplicate specific arms (same variant + same label twice)
2. No shadowed variants in wildcards (variant in wildcard that's
   later matched by a specific arm with a different label)
"""
from __future__ import annotations

import re


PATH = "crates/cross-cutting/rbac/src/value_objects.rs"


def find_match_blocks(lines):
    blocks = []
    for i, line in enumerate(lines):
        m = re.match(r"^(\s+)match self \{$", line)
        if m:
            indent = m.group(1)
            start = i
            depth = 1
            j = i + 1
            while j < len(lines) and depth > 0:
                for ch in lines[j]:
                    if ch == "{":
                        depth += 1
                    elif ch == "}":
                        depth -= 1
                        if depth == 0:
                            break
                if depth == 0:
                    break
                j += 1
            blocks.append((start, j, indent))
    return blocks


def parse_arms(lines, block_start, block_end, arm_indent):
    """Returns list of (rel_start, rel_end, [variants], label)."""
    arms = []
    j = block_start + 1
    while j < block_end:
        line = lines[j]
        rel = j - block_start

        m_single = re.match(
            r"^(\s+)((?:Self::\w+\s*\|\s*)*Self::\w+)\s*=>\s*\"([^\"]*)\",?\s*$",
            line,
        )
        if m_single and m_single.group(1) == arm_indent:
            variants = re.findall(r"Self::(\w+)", m_single.group(2))
            label = m_single.group(3)
            arms.append((rel, rel, variants, label))
            j += 1
            continue

        m_start = re.match(r"^(\s+)(Self::\w+(\s*\|\s*Self::\w+)*)\s*$", line)
        if m_start and m_start.group(1) == arm_indent and "=>" not in line:
            start_rel = rel
            variants = re.findall(r"Self::(\w+)", m_start.group(2))
            k = j + 1
            label = None
            while k < block_end:
                cont = lines[k].strip()
                if cont.startswith("=>"):
                    lm = re.search(r"\"([^\"]*)\"", cont)
                    if lm:
                        label = lm.group(1)
                    break
                elif cont.startswith("|"):
                    vs = re.findall(r"Self::(\w+)", cont)
                    variants.extend(vs)
                    if "=>" in cont:
                        lm = re.search(r"\"([^\"]*)\"", cont)
                        if lm:
                            label = lm.group(1)
                        break
                    k += 1
                else:
                    break
            if label is not None:
                arms.append((start_rel, k - block_start, variants, label))
                j = k + 1
                continue
            j += 1
            continue
        j += 1
    return arms


def fix_block(lines, block_start, block_end, indent):
    """For each match block, remove arms (or strip variants) that
    are redundant or shadowed by earlier arms."""
    arm_indent = indent + "    "
    arms = parse_arms(lines, block_start, block_end, arm_indent)
    if not arms:
        return 0

    # Track which variants have been seen across all previous arms.
    # First occurrence wins — subsequent matches of the same variant
    # (in any arm, with any label) are redundant.
    seen_variants = set()

    to_clear_abs = set()
    new_lines_rewrite = {}

    for idx, (s, e, variants, label) in enumerate(arms):
        # Step 1: strip within-arm duplicates (same variant twice in one arm)
        seen_in_arm = set()
        deduped = []
        for v in variants:
            if v in seen_in_arm:
                continue
            seen_in_arm.add(v)
            deduped.append(v)
        variants = deduped

        # Step 2: strip variants already covered by earlier arms
        kept = []
        for v in variants:
            if v in seen_variants:
                continue
            kept.append(v)

        if not kept:
            for k in range(s, e + 1):
                to_clear_abs.add(block_start + k)
            print(f"  L{block_start+s+1}: REMOVED arm ({len(deduped)} vars all covered)")
        elif len(kept) < len(variants):
            if len(kept) == 1:
                new_lines_rewrite[block_start + s] = (
                    f'{arm_indent}Self::{kept[0]} => "{label}",\n'
                )
            else:
                vstr = " | ".join(f"Self::{v}" for v in kept)
                new_lines_rewrite[block_start + s] = (
                    f'{arm_indent}{vstr} => "{label}",\n'
                )
            for k in range(s + 1, e + 1):
                to_clear_abs.add(block_start + k)
            print(f"  L{block_start+s+1}: REDUCED ({len(variants)}->{len(kept)})")

        for v in kept:
            seen_variants.add(v)

    # Apply rewrites and clearings
    for abs_idx, new_content in new_lines_rewrite.items():
        lines[abs_idx] = new_content
    for abs_idx in sorted(to_clear_abs, reverse=True):
        lines[abs_idx] = ""

    return len(to_clear_abs) + len(new_lines_rewrite)


def main():
    with open(PATH) as f:
        lines = f.read().split("\n")

    blocks = find_match_blocks(lines)
    print(f"Found {len(blocks)} match blocks")

    total = 0
    for block_start, block_end, indent in reversed(blocks):
        total += fix_block(lines, block_start, block_end, indent)

    print(f"\nTotal changes: {total}")
    new_src = "\n".join(lines)
    new_src = re.sub(r"\n{3,}", "\n\n", new_src)
    with open(PATH, "w") as f:
        f.write(new_src)


if __name__ == "__main__":
    main()
