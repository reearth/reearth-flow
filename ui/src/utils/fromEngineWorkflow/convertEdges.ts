import type { Edge, EngineReadyEdge } from "@flow/types";

export const convertEdges = (edges?: EngineReadyEdge[]) => {
  if (!edges) return [];
  const convertedEdges: Edge[] = edges.map((edge) => {
    const convertedEdge: Edge = {
      id: edge.id,
      source: edge.from,
      target: edge.to,
      sourceHandle: edge.fromPort,
      targetHandle: edge.toPort,
    };
    return convertedEdge;
  });
  return convertedEdges;
};
