import * as Y from "yjs";

import type { Edge, Node } from "@flow/types";

import { yNodeConstructor } from "../conversions";
import type { YNodesArray, YWorkflow } from "../types";

export function updateParentYWorkflow(
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
  // need to correct this type to use <Edge> instead of <unknown>
  const yParentEdges = yParentWorkflow?.get("edges") as Y.Array<unknown>;
  if (!yParentEdges) return;

  let hasUpdates = false;
  try {
    // Convert all edges to plain objects and update the one that needs changing
    const parentEdges = yParentEdges.toJSON() as Edge[];

    // Update the edges that are effected by the subworkflow node changes

    const newEdges = parentEdges.map((e) => {
      // Create a new object that we'll use to create a new YMap
      const newEdgeObj = { ...e };

      if (
        type === "source" &&
        e.source === currentWorkflowId &&
        e.sourceHandle === prevHandleName
      ) {
        hasUpdates = true;
        newEdgeObj.sourceHandle = params.routingPort;
      }

      if (
        type === "target" &&
        e.target === currentWorkflowId &&
        e.targetHandle === prevHandleName
      ) {
        hasUpdates = true;
        newEdgeObj.targetHandle = params.routingPort;
      }

      // This needs to be reviewed and changed
      const newYMap = new Y.Map();
      Object.entries(newEdgeObj).forEach(([key, value]) => {
        if (value !== undefined && value !== null) {
          newYMap.set(key, value);
        }
      });
      return newYMap;
    });

    // Only update if we made changes
    if (hasUpdates) {
      // Clear and recreate the entire edges array
      yParentEdges.delete(0, parentEdges.length);
      // Insert all edges at once as a single operation
      yParentEdges.insert(0, newEdges);
    }
  } catch (error) {
    console.error("Error cleaning up edges:", error);
  }
}

function cleanupRelatedEdges(
  currentWorkflowId: string,
  yParentWorkflow: YWorkflow,
  portName: string | undefined,
  type: "source" | "target",
) {
  if (!portName || !currentWorkflowId) return;

  const yParentEdges = yParentWorkflow?.get("edges") as Y.Array<unknown>;
  if (!yParentEdges || yParentEdges.length === 0) return;

  try {
    const edgesArray = Array.from(yParentEdges);

    // First convert all to plain objects and filter out edges to remove
    const remainingEdgeObjects = edgesArray
      .map((e) => (e as Y.Map<unknown>).toJSON() as Edge)
      .filter((edgeObj) => {
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

    // If we have any changes (length is different)
    if (remainingEdgeObjects.length !== edgesArray.length) {
      // Create fresh YMaps for all remaining edges
      const newEdges = remainingEdgeObjects.map((edgeObj) => {
        const newYMap = new Y.Map();
        Object.entries(edgeObj).forEach(([key, value]) => {
          if (value !== undefined && value !== null) {
            newYMap.set(key, value);
          }
        });
        return newYMap;
      });

      // Update the edges array
      yParentEdges.delete(0, edgesArray.length);
      if (newEdges.length > 0) {
        yParentEdges.insert(0, newEdges);
      }
    }
  } catch (error) {
    console.error("Error cleaning up edges:", error);
  }
}

const cleanupPseudoPortsHelper = (
  ports: { nodeId: string; portName: string }[],
  edgeType: "source" | "target",
  currentWorkflowId: string,
  yParentWorkflow: YWorkflow,
  nodeToDelete: Node,
) => {
  const portToRemove = ports.find((port) => port.nodeId === nodeToDelete.id);
  const updatedPorts = ports.filter((port) => port.nodeId !== nodeToDelete.id);

  if (portToRemove) {
    cleanupRelatedEdges(
      currentWorkflowId,
      yParentWorkflow,
      portToRemove.portName,
      edgeType,
    );
  }

  return updatedPorts;
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
    updatedSubworkflowNode.data.pseudoInputs = cleanupPseudoPortsHelper(
      updatedSubworkflowNode.data.pseudoInputs ?? [],
      "target",
      currentWorkflowId,
      yParentWorkflow,
      nodeToDelete,
    );
  }

  if (nodeToDelete.data.inputs?.length) {
    updatedSubworkflowNode.data.pseudoOutputs = cleanupPseudoPortsHelper(
      updatedSubworkflowNode.data.pseudoOutputs ?? [],
      "source",
      currentWorkflowId,
      yParentWorkflow,
      nodeToDelete,
    );
  }

  const newParentNodes = parentNodes.map((node) =>
    node.id === subworkflowNode.id ? updatedSubworkflowNode : node,
  );

  const newParentYNodes = newParentNodes.map((node) => yNodeConstructor(node));
  yParentNodes.delete(0, parentNodes.length);
  yParentNodes.insert(0, newParentYNodes);
}
