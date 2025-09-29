import type { EngineReadyGraph, Workflow } from "@flow/types";

import { convertEdges } from "./convertEdges";
import { convertNodes } from "./convertNodes";

export const createSubGraphs = (workflows: Workflow[]) => {
  const subGraphs = workflows.map((swf) => {
    const convertedNodes = convertNodes(swf.nodes);

    // Get IDs of enabled nodes to filter edges
    const enabledNodeIds = new Set(convertedNodes.map((node) => node.id));
    const convertedEdges = convertEdges(swf.edges, enabledNodeIds);

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
