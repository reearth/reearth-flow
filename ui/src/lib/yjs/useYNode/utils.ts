import type { Node } from "@flow/types";

import {
  reassembleEdge,
  yEdgeConstructor,
  yNodeConstructor,
} from "../conversions";
import type { YEdge, YEdgesArray, YNodesArray, YWorkflow } from "../types";

export function updateParentYWorkflowNode(
  currentWorkflowId: string,
  yParentWorkflow: YWorkflow,
  node: Node,
  params: any,
) {
  const yParentNodes = yParentWorkflow?.get("nodes") as YNodesArray | undefined;
  if (!yParentNodes) return;

  const parentNodes = yParentNodes.toJSON() as Node[];

  // Update the subworkflow node with the updated input/output
  const subworkflowNode = parentNodes.find((n) => n.id === currentWorkflowId);
  if (!subworkflowNode) return;

  const newPseudoPort = {
    nodeId: node.id,
    portName: params.routingPort,
  };

  const isRouterInput = node.data.outputs?.length;
  const isRouterOutput = node.data.inputs?.length;

  // Update pseudo ports and assign them back to the node
  const updatedPseudoPorts = isRouterInput
    ? updatePseudoPorts(
        subworkflowNode.data.pseudoInputs ?? [],
        newPseudoPort,
        currentWorkflowId,
        yParentWorkflow,
        "target",
      )
    : isRouterOutput
      ? updatePseudoPorts(
          subworkflowNode.data.pseudoOutputs ?? [],
          newPseudoPort,
          currentWorkflowId,
          yParentWorkflow,
          "source",
        )
      : null;

  if (!updatedPseudoPorts) return;

  const updatedSubworkflowNode: Node = {
    ...subworkflowNode,
    data: {
      ...subworkflowNode.data,
      [isRouterInput ? "pseudoInputs" : "pseudoOutputs"]: updatedPseudoPorts,
    },
  };

  const newParentNodes = parentNodes.map((node) =>
    node.id === updatedSubworkflowNode.id ? updatedSubworkflowNode : node,
  );

  const newParentYNodes = newParentNodes.map((node) => yNodeConstructor(node));

  yParentNodes.delete(0, parentNodes.length);
  yParentNodes.insert(0, newParentYNodes);
}

// Function to update pseudoInputs or pseudoOutputs
function updatePseudoPorts(
  pseudoPorts: { nodeId: string; portName: string }[],
  newPseudoPort: { nodeId: string; portName: string },
  currentWorkflowId: string,
  yParentWorkflow: YWorkflow,
  edgeType: "source" | "target",
) {
  const portIndex = pseudoPorts.findIndex(
    (port) => port.nodeId === newPseudoPort.nodeId,
  );
  const prevPseudoPort = portIndex !== -1 ? pseudoPorts[portIndex] : null;

  // If the pseudoInput/Output already exists, we want to update it. Otherwise, we want to add it.
  const updatedPseudoPorts =
    portIndex !== -1
      ? pseudoPorts.map((port, idx) =>
          idx === portIndex ? newPseudoPort : port,
        )
      : [...pseudoPorts, newPseudoPort];

  // Update ParentYWorkflow Edge effected by change in an exisiting pseudoInput/Output
  const yParentEdges = yParentWorkflow?.get("edges") as YEdgesArray | undefined;
  if (!yParentEdges) return updatedPseudoPorts;

  if (prevPseudoPort?.portName) {
    try {
      const edgeIndex = Array.from(yParentEdges).findIndex((e) => {
        const edgeObj = reassembleEdge(e as YEdge);
        return edgeType === "source"
          ? edgeObj.source === currentWorkflowId &&
              edgeObj.sourceHandle === prevPseudoPort.portName
          : edgeObj.target === currentWorkflowId &&
              edgeObj.targetHandle === prevPseudoPort.portName;
      });

      if (edgeIndex !== -1) {
        const currentEdge = reassembleEdge(
          yParentEdges.get(edgeIndex) as YEdge,
        );
        const updatedEdge = {
          ...currentEdge,
          [edgeType === "source" ? "sourceHandle" : "targetHandle"]:
            newPseudoPort.portName,
        };

        const newYEdge = yEdgeConstructor(updatedEdge);

        yParentEdges.delete(edgeIndex, 1);
        yParentEdges.insert(edgeIndex, [newYEdge]);
      }
    } catch (error) {
      console.error("Error updating edges:", error);
    }
  }

  return updatedPseudoPorts;
}

// Deletion and cleanup functions
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
        yParentWorkflow,
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
  yParentWorkflow: YWorkflow,
  portName: string,
  type: "source" | "target",
) {
  const yParentEdges = yParentWorkflow?.get("edges") as YEdgesArray | undefined;
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
