import isEqual from "lodash/isEqual";
import { useCallback } from "react";

import type { Node } from "@flow/types";

import { YNodesArray, YWorkflow } from "./workflowBuilder";

export default (workflow: YWorkflow) => {
  const handleNodesUpdate = useCallback(
    (newNodes: Node[]) => {
      const yNodes = workflow?.get("nodes") as YNodesArray | undefined;
      if (!yNodes) return;

      const n = yNodes.toJSON() as Node[];

      if (isEqual(n, newNodes)) return;

      yNodes.delete(0, n.length);
      yNodes.insert(0, newNodes);
    },
    [workflow],
  );
  return {
    handleNodesUpdate,
  };
};
