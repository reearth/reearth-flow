import type { Node } from "@flow/types";

import { reassembleEdge, yEdgeConstructor } from "../conversions";
import type { YEdge, YEdgesArray, YNodesArray, YWorkflow } from "../types";

import { getUpdatedPseudoPortsParam } from "./utils";

export function updateParentYWorkflowEdges(
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

export function removeEdgePort(
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
