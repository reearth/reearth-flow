import * as Y from "yjs";

import type { Edge, Node } from "@flow/types";

import {
  reassembleEdge,
  yEdgeConstructor,
  yNodeConstructor,
} from "../conversions";
import type { YEdge, YNodesArray, YWorkflow } from "../types";

export function updateParentYWorkflowNode(
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

  const updatedSubworkflowNode: Node = { ...subworkflowNode };

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

  const newParentYNode = newParentNodes.map((node) => yNodeConstructor(node));

  yParentNodes.delete(0, parentNodes.length);
  yParentNodes.insert(0, newParentYNode);
}

function updateParentYWorkflowEdges(
  currentWorkflowId: string,
  yParentWorkflow: YWorkflow,
  params: any,
  prevHandleName: string,
  type: "source" | "target",
) {
  if (!prevHandleName || !currentWorkflowId) return;

  const yParentEdges = yParentWorkflow?.get("edges") as Y.Array<unknown>;
  if (!yParentEdges) return;

  const parentEdges = yParentEdges.toJSON() as Edge[];
  let hasUpdates = false;

  const updatedEdges = parentEdges.map((e) => {
    if (
      type === "source" &&
      e.source === currentWorkflowId &&
      e.sourceHandle === prevHandleName
    ) {
      hasUpdates = true;
      return { ...e, sourceHandle: params.routingPort };
    }
    if (
      type === "target" &&
      e.target === currentWorkflowId &&
      e.targetHandle === prevHandleName
    ) {
      hasUpdates = true;
      return { ...e, targetHandle: params.routingPort };
    }
    return e;
  });

  // Only create YMaps if we actually made changes
  if (hasUpdates) {
    const newYMapEdges = updatedEdges.map((edge) => {
      const newYMap = new Y.Map();
      Object.entries(edge).forEach(([key, value]) => {
        if (value !== undefined && value !== null) {
          newYMap.set(key, value);
        }
      });
      return newYMap;
    });

    yParentEdges.delete(0, parentEdges.length);
    yParentEdges.insert(0, newYMapEdges);
  }
}

function removeEdgePort(
  currentWorkflowId: string,
  yParentWorkflow: YWorkflow,
  portName: string,
  type: "source" | "target",
) {
  const yParentEdges = yParentWorkflow?.get("edges") as Y.Array<unknown>;
  if (!yParentEdges || yParentEdges.length === 0) return;

  try {
    const edgesArray = yParentEdges.map((e) => reassembleEdge(e as YEdge));

    const updatedEdges = edgesArray.filter((edgeObj) => {
      if (type === "source") {
        return !(
          edgeObj.source === currentWorkflowId &&
          edgeObj.sourceHandle === portName
        );
      }
      return !(
        edgeObj.target === currentWorkflowId &&
        edgeObj.targetHandle === portName
      );
    });

    const newYEdges = updatedEdges.map((edgeObj) => yEdgeConstructor(edgeObj));

    yParentEdges.delete(0, edgesArray.length);
    yParentEdges.insert(0, newYEdges);
  } catch (error) {
    console.error("Error cleaning up edges:", error);
  }
}

const splitPorts = (
  ports: { nodeId: string; portName: string }[],
  nodeToDelete: Node,
) => {
  const portToRemove = ports.find((port) => port.nodeId === nodeToDelete.id);
  const portsToUpdate = ports.filter((port) => port.nodeId !== nodeToDelete.id);

  return { portToRemove, portsToUpdate };
};

export function cleanupPseudoPorts(
  currentWorkflowId: string,
  yParentWorkflow: YWorkflow,
  nodeToDelete: Node,
) {
  const yParentNodes = yParentWorkflow?.get("nodes") as YNodesArray | undefined;
  if (!yParentNodes) return;

  const parentNodes = yParentNodes.toJSON() as Node[];
  const subworkflowNode = parentNodes.find((n) => n.id === currentWorkflowId);
  if (!subworkflowNode) return;

  const updatedSubworkflowNode: Node = {
    ...subworkflowNode,
    data: {
      ...subworkflowNode.data,
    },
  };

  if (nodeToDelete.data.outputs?.length) {
    const { portToRemove, portsToUpdate } = splitPorts(
      updatedSubworkflowNode.data.pseudoInputs ?? [],
      nodeToDelete,
    );
    updatedSubworkflowNode.data.pseudoInputs = portsToUpdate;

    if (portToRemove) {
      removeEdgePort(
        currentWorkflowId,
        yParentWorkflow,
        portToRemove.portName,
        "target",
      );
    }
  }

  if (nodeToDelete.data.inputs?.length) {
    const { portToRemove, portsToUpdate } = splitPorts(
      updatedSubworkflowNode.data.pseudoOutputs ?? [],
      nodeToDelete,
    );
    updatedSubworkflowNode.data.pseudoOutputs = portsToUpdate;

    if (portToRemove) {
      removeEdgePort(
        currentWorkflowId,
        yParentWorkflow,
        portToRemove.portName,
        "source",
      );
    }
  }

  const newParentNodes = parentNodes.map((node) =>
    node.id === subworkflowNode.id ? updatedSubworkflowNode : node,
  );

  const newParentYNodes = newParentNodes.map((node) => yNodeConstructor(node));
  yParentNodes.delete(0, parentNodes.length);
  yParentNodes.insert(0, newParentYNodes);
}
