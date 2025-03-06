import type { Connection } from "@xyflow/react";

import type { Edge, Node } from "@flow/types";

export function isValidConnection(connection: Connection | Edge): boolean {
  return connection.source !== connection.target;
}

export function checkForReader(nodes: Node[] | undefined): boolean {
  if (!nodes) return false;
  return nodes.some((node) => node.type === "reader");
}
