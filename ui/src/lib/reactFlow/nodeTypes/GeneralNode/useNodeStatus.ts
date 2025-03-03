import type { NodeExecution } from "@flow/types";

export default () => {
  const nodeExecution: NodeExecution | undefined = {
    nodeId: "1",
    status: "running",
    startedAt: "2021-08-02T00:00:00Z",
    // completedAt: "2021-08-02T00:00:00Z",
    // intermediateDataUrl: "https://example.com",
  };

  return {
    nodeExecution,
  };
};
