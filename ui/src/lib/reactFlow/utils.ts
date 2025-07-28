import type { Connection } from "@xyflow/react";

import type { Edge } from "@flow/types";

export function isValidConnection(connection: Connection | Edge): boolean {
  return connection.source !== connection.target;
}
