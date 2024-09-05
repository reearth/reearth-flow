import isEqual from "lodash-es/isEqual";
import { useCallback } from "react";

import type { Node } from "@flow/types";

import { YNodesArray, YWorkflow } from "./workflowBuilder";

export default ({
  currentYWorkflow,
  undoTrackerActionWrapper,
  handleWorkflowsRemove,
}: {
  currentYWorkflow: YWorkflow;
  undoTrackerActionWrapper: (callback: () => void) => void;
  handleWorkflowsRemove: (workflowId: string[]) => void;
}) => {
  const handleNodesUpdate = useCallback(
    (newNodes: Node[]) =>
      undoTrackerActionWrapper(() => {
        const yNodes = currentYWorkflow?.get("nodes") as
          | YNodesArray
          | undefined;
        if (!yNodes) return;

        const n = yNodes.toJSON() as Node[];

        if (isEqual(n, newNodes)) return;

        if (newNodes.length < n.length) {
          const idsToBeRemoved = nodesToBeRemoved(n, newNodes)
            .filter((n) => n.type === "subworkflow")
            .map((n) => n.id);

          if (idsToBeRemoved.length > 0) {
            handleWorkflowsRemove(idsToBeRemoved);
          }
        }

        yNodes.delete(0, n.length);
        yNodes.insert(0, newNodes);
      }),
    [currentYWorkflow, undoTrackerActionWrapper, handleWorkflowsRemove],
  );
  return {
    handleNodesUpdate,
  };
};

function nodesToBeRemoved(oldNodes: Node[], NewNodes: Node[]) {
  const isInArray = (node: Node, nodeArray: Node[]) =>
    nodeArray.some((item) => item.id === node.id);
  const removedNodes = oldNodes.filter((n) => !isInArray(n, NewNodes));
  return removedNodes;
}
