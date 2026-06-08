## graphify

This project has a graphify knowledge graph at schoolify/graphify-out/.

Rules:
- Always use graphify when exploring the schoolify codebase
- Before answering architecture or codebase questions, read schoolify/graphify-out/GRAPH_REPORT.md for god nodes and community structure
- If schoolify/graphify-out/wiki/index.md exists, navigate it instead of reading raw files
- For cross-module "how does X relate to Y" questions, prefer `cd schoolify && graphify query "<question>"`, `cd schoolify && graphify path "<A>" "<B>"`, or `cd schoolify && graphify explain "<concept>"` over grep — these traverse the graph's EXTRACTED + INFERRED edges instead of scanning files
- After modifying code files in this session, run `cd schoolify && graphify update .` to keep the graph current (AST-only, no API cost)
