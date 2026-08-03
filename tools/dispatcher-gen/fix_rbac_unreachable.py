#!/usr/bin/env python3
"""
Fix unreachable_patterns in crates/cross-cutting/rbac/src/value_objects.rs.

Root cause: wildcard match arms include variants that are ALSO matched
by earlier arms with different labels. The compiler correctly flags
the earlier wildcard as dead code (since the later arm never executes
for those variants).

Algorithm:
1. Find all `match self { ... }` blocks
2. Parse arms, handling both single-line and multi-line wildcards
3. Build a map: variant -> [(arm_idx, label)] for ALL arms
4. For each wildcard arm, find which variants have a DIFFERENT
   label in some other arm
5. Strip those variants from the wildcard (or remove the whole arm)

Key insight: The FIRST line of a multi-line arm has format
"            Self::X" (no |), and continuation lines start with |.
The regex must accept both forms.
"""
from __future__ import annotations

import re
import sys


PATH = "crates/cross-cutting/rbac/src/value_objects.rs"


def find_match_blocks(lines):
    """Yield (start, end, indent) for each `match self { ... }` block."""
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
    """Parse arms. Returns list of (rel_start, rel_end, variants, label)."""
    arms = []
    j = block_start + 1
    while j < block_end:
        line = lines[j]
        rel = j - block_start

        # Single-line arm: "            Self::X | Self::Y => \"label\","
        m = re.match(
            r"^(\s+)((?:Self::\w+\s*\|\s*)*Self::\w+)\s*=>\s*\"([^\"]*)\",?\s*$",
            line,
        )
        if m and m.group(1) == arm_indent:
            variants = re.findall(r"Self::(\w+)", m.group(2))
            label = m.group(3)
            arms.append((rel, rel, variants, label))
            j += 1
            continue

        # Multi-line arm: first line is "            Self::X" (possibly with |)
        # Continuation lines start with "| Self::Y". Final line has "=> \"label\","
        m2 = re.match(r"^(\s+)((?:Self::\w+\s*\|?\s*)*Self::\w+)\s*$", line)
        if m2 and m2.group(1) == arm_indent and "=>" not in line:
            start_rel = rel
            variants = re.findall(r"Self::(\w+)", line)
            k = j + 1
            label = None
            while k < block_end:
                cont = lines[k].strip()
                if cont.startswith("|"):
                    variants.extend(re.findall(r"Self::(\w+)", cont))
                    k += 1
                elif cont.startswith("=>"):
                    lm = re.search(r"\"([^\"]*)\"", cont)
                    if lm:
                        label = lm.group(1)
                    break
                else:
                    break
            if label is not None:
                arms.append((start_rel, k - block_start, variants, label))
                j = k + 1
                continue
        j += 1
    return arms


def fix_block(lines, block_start, block_end, indent):
    """Fix one match block. Returns number of lines cleared."""
    arm_indent = indent + "    "
    arms = parse_arms(lines, block_start, block_end, arm_indent)
    if not arms:
        return 0

    # Build map: variant -> [(arm_idx, label)] for ALL arms
    variant_arms = {}
    for idx, (s, e, variants, label) in enumerate(arms):
        for v in variants:
            variant_arms.setdefault(v, []).append((idx, label))

    # For each wildcard arm, find shadowed variants
    to_clear_abs = set()
    for idx, (s, e, variants, label) in enumerate(arms):
        if len(variants) < 2:
            continue
        # A variant is shadowed if some OTHER arm matches it with a different label
        shadowed = set()
        for v in variants:
            for other_idx, other_label in variant_arms[v]:
                if other_idx != idx and other_label != label:
                    shadowed.add(v)
                    break
        if not shadowed:
            continue
        kept = [v for v in variants if v not in shadowed]
        if not kept:
            # All variants shadowed — remove the entire arm
            for k in range(s, e + 1):
                to_clear_abs.add(block_start + k)
            print(f"  L{block_start+s+1}: REMOVED wildcard [{len(variants)} vars]")
        elif len(kept) == 1:
            # Becomes a specific arm
            lines[block_start + s] = (
                f'{arm_indent}Self::{kept[0]} => "{label}",\n'
            )
            for k in range(s + 1, e + 1):
                to_clear_abs.add(block_start + k)
            print(f"  L{block_start+s+1}: -> SPECIFIC ({len(variants)}->1)")
        else:
            # Reduce wildcard
            vstr = " | ".join(f"Self::{v}" for v in kept)
            lines[block_start + s] = (
                f'{arm_indent}{vstr} => "{label}",\n'
            )
            for k in range(s + 1, e + 1):
                to_clear_abs.add(block_start + k)
            print(f"  L{block_start+s+1}: REDUCED ({len(variants)}->{len(kept)})")

    # Apply removals (reverse to preserve indices)
    for abs_idx in sorted(to_clear_abs, reverse=True):
        lines[abs_idx] = ""
    return len(to_clear_abs)


def main():
    with open(PATH) as f:
        lines = f.read().split("\n")

    blocks = find_match_blocks(lines)
    print(f"Found {len(blocks)} match blocks")

    total_cleared = 0
    # Process in REVERSE order so line indices remain stable
    for block_start, block_end, indent in reversed(blocks):
        cleared = fix_block(lines, block_start, block_end, indent)
        total_cleared += cleared

    print(f"\nTotal lines cleared: {total_cleared}")

    new_src = "\n".join(lines)
    new_src = re.sub(r"\n{3,}", "\n\n", new_src)
    with open(PATH, "w") as f:
        f.write(new_src)


if __name__ == "__main__":
    main()
