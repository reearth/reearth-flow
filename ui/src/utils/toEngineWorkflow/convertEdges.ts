import { DEFAULT_EDGE_PORT } from "@flow/global-constants";
import type { Edge, EngineReadyEdge } from "@flow/types";

export const convertEdges = (edges?: Edge[], enabledNodeIds?: Set<string>) => {
  if (!edges) return [];

  // Filter out edges that connect to disabled nodes to ensure the workflow engine only processes valid connections
  const validEdges = enabledNodeIds
    ? edges.filter(
        (edge) =>
          enabledNodeIds.has(edge.source) && enabledNodeIds.has(edge.target),
      )
    : edges;

  const convertedEdges: EngineReadyEdge[] = validEdges.map((edge) => {
    return {
      id: edge.id,
      from: edge.source,
      to: edge.target,
      fromPort: edge.sourceHandle ?? DEFAULT_EDGE_PORT,
      toPort: edge.targetHandle ?? DEFAULT_EDGE_PORT,
    };
  });
  return convertedEdges;
};
