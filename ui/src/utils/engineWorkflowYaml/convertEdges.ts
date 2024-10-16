import type { Edge, EngineReadyEdge } from "@flow/types";

export const convertEdges = (edges?: Edge[]) => {
  if (!edges) return [];
  const convertedEdges: EngineReadyEdge[] = edges.map((edge) => {
    return {
      id: edge.id,
      from: edge.source,
      to: edge.target,
      fromPort: edge.sourceHandle ?? "default",
      toPort: edge.targetHandle ?? "default",
    };
  });
  return convertedEdges;
};
