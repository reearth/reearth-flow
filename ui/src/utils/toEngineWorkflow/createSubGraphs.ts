import type { EngineReadyGraph, Workflow } from "@flow/types";

import { convertEdges } from "./convertEdges";
import { convertNodes } from "./convertNodes";

export const createSubGraphs = (workflows: Workflow[]) => {
  const subGraphs = workflows.map((swf) => {
    const convertedNodes = convertNodes(swf.nodes);
    const convertedEdges = convertEdges(swf.edges);

    const subGraph: EngineReadyGraph = {
      id: swf.id,
      name: swf.name ?? "undefined-graph",
      nodes: convertedNodes,
      edges: convertedEdges,
    };

    return subGraph;
  });
  return subGraphs;
};
