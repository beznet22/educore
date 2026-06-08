// graphify OpenCode plugin
// Injects a knowledge graph reminder before bash tool calls when the graph exists.
import { existsSync } from "fs";
import { join } from "path";

export const GraphifyPlugin = async ({ directory }) => {
  let reminded = false;

  // Check for graph in current directory or schoolify subdirectory
  const graphPaths = [
    join(directory, "graphify-out", "graph.json"),
    join(directory, "schoolify", "graphify-out", "graph.json"),
  ];

  return {
    "tool.execute.before": async (input, output) => {
      if (reminded) return;
      
      // Find which graph path exists
      const graphPath = graphPaths.find(p => existsSync(p));
      if (!graphPath) return;
      
      // Determine the relative path prefix for AGENTS.md instructions
      const relativePath = graphPath.includes(join("schoolify", "graphify-out")) 
        ? "schoolify/graphify-out" 
        : "graphify-out";

      if (input.tool === "bash") {
        output.args.command =
          `echo "[graphify] Knowledge graph available. Read ${relativePath}/GRAPH_REPORT.md for god nodes and architecture context before searching files." && ` +
          output.args.command;
        reminded = true;
      }
    },
  };
};
