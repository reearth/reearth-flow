export type NodeStatus = "pending" | "running" | "succeeded" | "failed";

export type NodeExecution = {
  nodeId: string;
  status: NodeStatus;
  startedAt?: string;
  completedAt?: string;
  intermediateDataUrl?: string;
};
