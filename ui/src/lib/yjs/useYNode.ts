import isEqual from "lodash-es/isEqual";
import { useCallback } from "react";
import * as Y from "yjs";

import type { Edge, Node } from "@flow/types";

import { YNodesArray, YWorkflow } from "./utils";

export default ({
  currentYWorkflow,
  yWorkflows,
  rawWorkflows,
  undoTrackerActionWrapper,
  handleWorkflowsRemove,
}: {
  currentYWorkflow: YWorkflow;
  yWorkflows: Y.Array<YWorkflow>;
  rawWorkflows: Record<string, string | Node[] | Edge[]>[];
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

        // If one or more nodes are deleted
        if (newNodes.length < n.length) {
          const idsToBeRemoved = nodesToBeRemoved(n, newNodes).map((n) => n.id);

          if (idsToBeRemoved.length > 0) {
            handleWorkflowsRemove(idsToBeRemoved);
          }
          // TODO:
          // Currently here we are doing "cleanup" to
          // remove the subworkflow nodes that are not used anymore.
          // What we want is to have a cleanup function
          // that does this removal but also to update
          // any subworkflow nodes' pseudoInputs and pseudoOutputs
          // that are effected by the removal of the subworkflow node. @KaWaite
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

        // if params.routingPort && currentWorkflow is a subworkflow.
        if (params.routingPort) {
          updateParentYWorkflow(
            rawWorkflows,
            yWorkflows,
            currentYWorkflow,
            node,
            params,
          );
        }

        const updatedNode: Node = { ...node, data: { ...node.data, params } };
        const newNodes = [...nodes];
        newNodes.splice(nodeIndex, 1, updatedNode);

        yNodes.delete(0, nodes.length);
        yNodes.insert(0, newNodes);
      });
    },
    [currentYWorkflow, rawWorkflows, yWorkflows, undoTrackerActionWrapper],
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

function updateParentYWorkflow(
  rawWorkflows: Record<string, string | Node[] | Edge[]>[],
  yWorkflows: Y.Array<YWorkflow>,
  currentYWorkflow: YWorkflow,
  node: Node,
  params: any,
) {
  // Find which workflow the current workflow is used
  const workflowIndex = rawWorkflows.findIndex((w) => {
    const nodes = w.nodes as Node[];
    return nodes.some(
      (n) => n.id === (currentYWorkflow.get("id")?.toJSON() as string),
    );
  });
  const yParentWorkflow = yWorkflows.get(workflowIndex);

  const currentWorkflowId = currentYWorkflow.get("id")?.toJSON() as string;

  // From here we are updating pseudoInputs and pseudoOutputs.
  // These only exist on subworkflow nodes.
  updateParentYWorkflowNode(currentWorkflowId, yParentWorkflow, node, params);
}

function updateParentYWorkflowNode(
  currentWorkflowId: string,
  yParentWorkflow: YWorkflow,
  node: Node,
  params: any,
) {
  // Find the subworkflow node in that workflow.
  const yParentNodes = yParentWorkflow?.get("nodes") as YNodesArray | undefined;
  if (!yParentNodes) return;

  const parentNodes = yParentNodes.toJSON() as Node[];

  // Update the subworkflow node with the updated input/output
  const subworkflowNode = parentNodes.find((n) => n.id === currentWorkflowId);
  if (!subworkflowNode) return;

  const updatedSubworkflowNode: Node = {
    ...subworkflowNode,
    data: {
      ...subworkflowNode.data,
    },
  };

  const newPseudoPort = {
    nodeId: node.id,
    portName: params.routingPort,
  };

  // Here we want to update pseudoInputs if the node is a RouterInput (RouterInputs only have outputs as default)
  if (node.data.outputs?.length) {
    const previousPseudoInputs = updatedSubworkflowNode.data.pseudoInputs ?? [];
    const updatedPseudoInputs = [...previousPseudoInputs];

    const toBeUpdatedPseudoInputIndex = updatedPseudoInputs.findIndex(
      (upi) => upi.nodeId === node.id,
    );

    // If the pseudoInput already exists, we want to update it. Otherwise, we want to add it.
    if (toBeUpdatedPseudoInputIndex !== -1) {
      updatedPseudoInputs.splice(toBeUpdatedPseudoInputIndex, 1, newPseudoPort);
    } else {
      updatedPseudoInputs.push(newPseudoPort);
    }
    updatedSubworkflowNode.data.pseudoInputs = updatedPseudoInputs;

    // Update edges effected
    updateParentYWorkflowEdges(
      currentWorkflowId,
      yParentWorkflow,
      params,
      previousPseudoInputs[toBeUpdatedPseudoInputIndex]?.portName,
      "target",
    );

    // Here we want to update pseudoOutputs if the node is a RouterOutput (RouterOutputs only have inputs as default)
  } else if (node.data.inputs?.length) {
    const previousPseudoOutputs =
      updatedSubworkflowNode.data.pseudoOutputs ?? [];
    const updatedPseudoOutputs = [...previousPseudoOutputs];

    const toBeUpdatedPseudoOutputIndex = updatedPseudoOutputs.findIndex(
      (upi) => upi.nodeId === node.id,
    );

    // If the pseudoOutput already exists, we want to update it. Otherwise, we want to add it.
    if (toBeUpdatedPseudoOutputIndex !== -1) {
      updatedPseudoOutputs.splice(
        toBeUpdatedPseudoOutputIndex,
        1,
        newPseudoPort,
      );
    } else {
      updatedPseudoOutputs.push(newPseudoPort);
    }
    updatedSubworkflowNode.data.pseudoOutputs = updatedPseudoOutputs;

    // Update edges effected
    updateParentYWorkflowEdges(
      currentWorkflowId,
      yParentWorkflow,
      params,
      previousPseudoOutputs[toBeUpdatedPseudoOutputIndex]?.portName,
      "source",
    );
  }

  const newParentNodes = [...parentNodes];
  newParentNodes.splice(
    parentNodes.indexOf(subworkflowNode),
    1,
    updatedSubworkflowNode,
  );

  yParentNodes.delete(0, parentNodes.length);
  yParentNodes.insert(0, newParentNodes);
}

function updateParentYWorkflowEdges(
  currentWorkflowId: string,
  yParentWorkflow: YWorkflow,
  params: any,
  prevHandleName: string,
  type: "source" | "target",
) {
  const yParentEdges = yParentWorkflow?.get("edges") as
    | Y.Array<Edge>
    | undefined;
  if (!yParentEdges) return;

  const parentEdges = yParentEdges.toJSON() as Edge[];
  console.log("params", params);

  // Update the edges that are effected by the subworkflow node changes
  const updatedEdges = parentEdges.map((e) => {
    if (
      e.source === currentWorkflowId &&
      type === "source" &&
      e.sourceHandle === prevHandleName
    ) {
      return { ...e, sourceHandle: params.routingPort };
    }
    if (
      e.target === currentWorkflowId &&
      type === "target" &&
      e.targetHandle === prevHandleName
    ) {
      return { ...e, targetHandle: params.routingPort };
    }
    return e;
  });

  yParentEdges.delete(0, parentEdges.length);
  yParentEdges.insert(0, updatedEdges);
}
