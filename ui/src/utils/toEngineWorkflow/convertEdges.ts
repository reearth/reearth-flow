import { DEFAULT_EDGE_PORT } from "@flow/global-constants";
import type { Edge, EngineReadyEdge } from "@flow/types";

export const convertEdges = (edges?: Edge[]) => {
  if (!edges) return [];
  const convertedEdges: EngineReadyEdge[] = edges.map((edge) => {
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
