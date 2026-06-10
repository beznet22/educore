#!/usr/bin/env bash
# tools/scripts/check-graph-freshness.sh
#
# 5th validation criterion (per docs/build-plan.md § No-Gaps Gates).
# Verifies that graphify-out/GRAPH_REPORT.md is no older than the
# latest commit that touched crates/, docs/, or migrations/.
# A stale graph is a sign that the local `graphify hook install`
# is broken or has been bypassed.
#
# This script does NOT regenerate the graph (the local hook does
# that on commit). It only checks freshness.

set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
GRAPH_REPORT="$REPO_ROOT/graphify-out/GRAPH_REPORT.md"

if [ ! -f "$GRAPH_REPORT" ]; then
    echo "FAIL: $GRAPH_REPORT does not exist."
    echo "  Hint: run \`graphify .\` from the repo root to generate it,"
    echo "  then \`graphify hook install\` to set up the auto-rebuild."
    exit 1
fi

# Find the latest commit that touched any of the source dirs
LATEST_SRC_COMMIT=$(
    git -C "$REPO_ROOT" log -1 --format=%ct \
        -- crates/ docs/ migrations/ .graphifyignore 2>/dev/null || echo 0
)

# Find the mtime of the graph report
GRAPH_MTIME=$(stat -c %Y "$GRAPH_REPORT" 2>/dev/null || echo 0)

if [ "$GRAPH_MTIME" -lt "$LATEST_SRC_COMMIT" ]; then
    echo "FAIL: $GRAPH_REPORT is older than the latest source change."
    echo "  Latest source commit: $(date -d @$LATEST_SRC_COMMIT '+%Y-%m-%d %H:%M:%S' 2>/dev/null || echo $LATEST_SRC_COMMIT)"
    echo "  Graph mtime:          $(date -d @$GRAPH_MTIME '+%Y-%m-%d %H:%M:%S' 2>/dev/null || echo $GRAPH_MTIME)"
    echo "  Hint: run \`graphify .\` to refresh the graph, then commit it."
    exit 1
fi

echo "OK: graphify-out/GRAPH_REPORT.md is fresh."
echo "  Latest source commit: $(date -d @$LATEST_SRC_COMMIT '+%Y-%m-%d %H:%M:%S' 2>/dev/null || echo $LATEST_SRC_COMMIT)"
echo "  Graph mtime:          $(date -d @$GRAPH_MTIME '+%Y-%m-%d %H:%M:%S' 2>/dev/null || echo $GRAPH_MTIME)"
exit 0
