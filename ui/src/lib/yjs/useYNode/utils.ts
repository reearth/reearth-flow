import type { Node, PseudoPort } from "@flow/types";

import {
  reassembleEdge,
  yEdgeConstructor,
  yNodeConstructor,
} from "../conversions";
import type { YEdge, YEdgesArray, YNodesArray, YWorkflow } from "../types";

export function updateParentYWorkflow(
  currentWorkflowId: string,
  parentYNodes: YNodesArray,
  parentYEdges: YEdgesArray,
  prevNode: Node,
  newParams: any,
) {
  const newPseudoPort: PseudoPort = {
    nodeId: prevNode.id,
    portName: newParams.routingPort,
  };

  updateParentYWorkflowEdges(
    currentWorkflowId,
    parentYNodes,
    parentYEdges,
    prevNode,
    newPseudoPort,
  );

  updateParentYWorkflowNode(
    currentWorkflowId,
    parentYNodes,
    prevNode,
    newPseudoPort,
  );
}

function updateParentYWorkflowNode(
  currentWorkflowId: string,
  parentYNodes: YNodesArray,
  prevNode: Node,
  newPseudoPort: PseudoPort,
) {
  const parentNodes = parentYNodes.toJSON() as Node[];

  // Update the subworkflow node with the updated input/output
  const parentNodeIndex = parentNodes.findIndex(
    (n) => n.id === currentWorkflowId,
  );
  const subworkflowParentNode = parentNodes[parentNodeIndex];

  if (!subworkflowParentNode) return;

  const isRouterInput = prevNode.data.outputs?.length;
  const isRouterOutput = prevNode.data.inputs?.length;

  const routerType = isRouterInput
    ? "pseudoInputs"
    : isRouterOutput
      ? "pseudoOutputs"
      : undefined;
  if (!routerType) return;

  const prevPseudoPorts = subworkflowParentNode.data[routerType];

  if (!prevPseudoPorts) return;

  const updatedPseudoPorts = getUpdatedPseudoPortsParam(
    prevPseudoPorts,
    newPseudoPort,
  );

  if (!updatedPseudoPorts) return;

  const updatedSubworkflowParentNode: Node = {
    ...subworkflowParentNode,
    data: {
      ...subworkflowParentNode.data,
      [routerType]: updatedPseudoPorts,
    },
  };

  const newParentNodes = parentNodes.map((node) =>
    node.id === updatedSubworkflowParentNode.id
      ? updatedSubworkflowParentNode
      : node,
  );

  const newParentYNodes = newParentNodes.map((node) => yNodeConstructor(node));

  parentYNodes.delete(0, parentNodes.length);
  parentYNodes.insert(0, newParentYNodes);
}

function updateParentYWorkflowEdges(
  currentWorkflowId: string,
  parentYNodes: YNodesArray,
  parentYEdges: YEdgesArray,
  prevNode: Node,
  newPseudoPort: { nodeId: string; portName: string },
) {
  const parentNodes = parentYNodes.toJSON() as Node[];

  // Update the subworkflow node with the updated input/output
  const subworkflowParentNode = parentNodes.find(
    (n) => n.id === currentWorkflowId,
  );
  if (!subworkflowParentNode) return;

  const isRouterInput = prevNode.data.outputs?.length;
  const isRouterOutput = prevNode.data.inputs?.length;

  const routerType = isRouterInput
    ? "pseudoInputs"
    : isRouterOutput
      ? "pseudoOutputs"
      : undefined;
  if (!routerType) return;

  const prevPseudoPorts = subworkflowParentNode.data[routerType];

  if (!prevPseudoPorts) return;

  const updatedPseudoPorts = getUpdatedPseudoPortsParam(
    prevPseudoPorts,
    newPseudoPort,
  );

  // Update ParentYWorkflow Edge effected by change in an exisiting pseudoInput/Output
  try {
    const edgeType = isRouterInput ? "target" : "source";
    const prevPseudoPort = prevPseudoPorts.find(
      (port) => port.nodeId === newPseudoPort.nodeId,
    );
    if (!prevPseudoPort) return;

    const edgeIndex = Array.from(parentYEdges).findIndex((e) => {
      const edgeObj = reassembleEdge(e as YEdge);
      return edgeType === "source"
        ? edgeObj.source === currentWorkflowId &&
            edgeObj.sourceHandle === prevPseudoPort.portName
        : edgeObj.target === currentWorkflowId &&
            edgeObj.targetHandle === prevPseudoPort.portName;
    });

    if (edgeIndex !== -1) {
      const currentEdge = reassembleEdge(parentYEdges.get(edgeIndex) as YEdge);
      const updatedEdge = {
        ...currentEdge,
        [edgeType === "source" ? "sourceHandle" : "targetHandle"]:
          newPseudoPort.portName,
      };

      const newYEdge = yEdgeConstructor(updatedEdge);

      parentYEdges.delete(edgeIndex, 1);
      parentYEdges.insert(edgeIndex, [newYEdge]);
    }
  } catch (error) {
    console.error("Error updating edges:", error);
  }

  return updatedPseudoPorts;
}

// Function to update and return pseudoInputs or pseudoOutputs
function getUpdatedPseudoPortsParam(
  prevPseudoPorts: PseudoPort[],
  newPseudoPort: PseudoPort,
): PseudoPort[] {
  const portIndex = prevPseudoPorts.findIndex(
    (port) => port.nodeId === newPseudoPort.nodeId,
  );

  // If the pseudoInput/Output already exists, we want to update it. Otherwise, we want to add it.
  const updatedPseudoPorts =
    portIndex !== -1
      ? prevPseudoPorts.map((port, idx) =>
          idx === portIndex ? newPseudoPort : port,
        )
      : [...prevPseudoPorts, newPseudoPort];

  return updatedPseudoPorts;
}

// Deletion and cleanup functions
export function cleanupPseudoPorts(
  currentWorkflowId: string,
  parentYWorkflow: YWorkflow,
  nodeToDelete: Node,
) {
  const parentYNodes = parentYWorkflow?.get("nodes") as YNodesArray | undefined;
  if (!parentYNodes) return;

  const parentNodes = parentYNodes.toJSON() as Node[];

  const subworkflowNode = parentNodes.find((n) => n.id === currentWorkflowId);
  if (!subworkflowNode) return;

  const updatedSubworkflowNode: Node = { ...subworkflowNode };

  const isRouterInput = nodeToDelete.data.outputs?.length;
  const isRouterOutput = nodeToDelete.data.inputs?.length;

  if (isRouterInput) {
    const { portToRemove, portsToUpdate } = splitPorts(
      updatedSubworkflowNode.data.pseudoInputs ?? [],
      nodeToDelete,
    );
    updatedSubworkflowNode.data.pseudoInputs = portsToUpdate;

    if (portToRemove) {
      removeEdgePort(
        currentWorkflowId,
        parentYWorkflow,
        portToRemove.portName,
        "target",
      );
    }
  }

  if (isRouterOutput) {
    const { portToRemove, portsToUpdate } = splitPorts(
      updatedSubworkflowNode.data.pseudoOutputs ?? [],
      nodeToDelete,
    );
    updatedSubworkflowNode.data.pseudoOutputs = portsToUpdate;

    if (portToRemove) {
      removeEdgePort(
        currentWorkflowId,
        parentYWorkflow,
        portToRemove.portName,
        "source",
      );
    }
  }

  const newParentNodes = parentNodes.map((node) =>
    node.id === subworkflowNode.id ? updatedSubworkflowNode : node,
  );

  const newParentYNodes = newParentNodes.map((node) => yNodeConstructor(node));
  parentYNodes.delete(0, parentNodes.length);
  parentYNodes.insert(0, newParentYNodes);
}

function splitPorts(
  ports: { nodeId: string; portName: string }[],
  nodeToDelete: Node,
) {
  const portToRemove = ports.find((port) => port.nodeId === nodeToDelete.id);
  const portsToUpdate = ports.filter((port) => port.nodeId !== nodeToDelete.id);

  return { portToRemove, portsToUpdate };
}

function removeEdgePort(
  currentWorkflowId: string,
  parentYWorkflow: YWorkflow,
  portName: string,
  type: "source" | "target",
) {
  const parentYEdges = parentYWorkflow?.get("edges") as YEdgesArray | undefined;
  if (!parentYEdges || parentYEdges.length === 0) return;

  try {
    const edgesArray = parentYEdges.map((e) => reassembleEdge(e as YEdge));

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

    parentYEdges.delete(0, edgesArray.length);
    parentYEdges.insert(0, newYEdges);
  } catch (error) {
    console.error("Error cleaning up edges:", error);
  }
}
