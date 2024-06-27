import type { Edge as ReactFlowEdge } from "@xyflow/react";

import type { Status } from "./shared";

export type EdgeData = {
  status?: Status;
};

export type Edge = ReactFlowEdge<EdgeData>;
