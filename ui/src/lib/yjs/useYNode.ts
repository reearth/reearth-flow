import isEqual from "lodash-es/isEqual";
import { useCallback } from "react";

import type { Node } from "@flow/types";

import { YNodesArray, YWorkflow } from "./utils";

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
          const idsToBeRemoved = nodesToBeRemoved(n, newNodes).map((n) => n.id);

          if (idsToBeRemoved.length > 0) {
            handleWorkflowsRemove(idsToBeRemoved);
          }
        }

        yNodes.delete(0, n.length);
        yNodes.insert(0, newNodes);
      }),
    [currentYWorkflow, undoTrackerActionWrapper, handleWorkflowsRemove],
  );

  const handleNodeParamsUpdate = useCallback(
    (nodeId: string, params: any) => {
      undoTrackerActionWrapper(() => {
        const yNodes = currentYWorkflow?.get("nodes") as
          | YNodesArray
          | undefined;
        if (!yNodes) return;

        const nodes = yNodes.toJSON() as Node[];

        const nodeIndex = nodes.findIndex((n) => n.id === nodeId);
        const node = nodes[nodeIndex];

        if (!node) return;

        const updatedNode: Node = { ...node, data: { ...node.data, params } };
        const newNodes = [...nodes];
        newNodes.splice(nodeIndex, 1, updatedNode);

        yNodes.delete(0, nodes.length);
        yNodes.insert(0, newNodes);
      });
    },
    [currentYWorkflow, undoTrackerActionWrapper],
  );

  return {
    handleNodesUpdate,
    handleNodeParamsUpdate,
  };
};

function nodesToBeRemoved(oldNodes: Node[], newNodes: Node[]) {
  const isInArray = (node: Node, nodeArray: Node[]) =>
    nodeArray.some((item) => item.id === node.id);
  const removedNodes = oldNodes.filter((n) => !isInArray(n, newNodes));
  return removedNodes;
}
