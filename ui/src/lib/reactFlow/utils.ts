import type { Connection } from "@xyflow/react";

import type { Edge } from "@flow/types";

export function isValidConnection(connection: Connection | Edge): boolean {
  // Prevent self-connection
  return connection.source !== connection.target;
}

export function isExistingConnection(edges: Edge[]) {
  return (connection: Connection | Edge): boolean => {
    // Prevents duplicate connections
    const existingConnection = edges.find(
      (edge: Edge) =>
        edge.source === connection.source &&
        edge.target === connection.target &&
        edge.sourceHandle === connection.sourceHandle &&
        edge.targetHandle === connection.targetHandle,
    );

    return !existingConnection;
  };
}
