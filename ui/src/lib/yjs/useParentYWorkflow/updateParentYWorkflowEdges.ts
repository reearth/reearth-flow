import type { Node } from "@flow/types";

import { reassembleEdge, yEdgeConstructor } from "../conversions";
import type { YEdge, YEdgesMap, YNodesMap, YWorkflow } from "../types";

import { getUpdatedPseudoPortsParam } from "./utils";

export function updateParentYWorkflowEdges(
  currentWorkflowId: string,
  parentYNodes: YNodesMap,
  parentYEdges: YEdgesMap,
  prevNode: Node,
  newPseudoPort: { nodeId: string; portName: string },
) {
  const parentNodes = Object.values(parentYNodes.toJSON()) as Node[];

  // Update the subworkflow node with the updated input/output
  const subworkflowParentNode = parentNodes.find(
    (n) => n.data.subworkflowId === currentWorkflowId,
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

    const [yEdgeId, yEdge] =
      Array.from(parentYEdges).find(([, e]) => {
        const edgeObj = reassembleEdge(e as YEdge);
        return edgeType === "source"
          ? edgeObj.source === subworkflowParentNode.id &&
              edgeObj.sourceHandle === prevPseudoPort.portName
          : edgeObj.target === subworkflowParentNode.id &&
              edgeObj.targetHandle === prevPseudoPort.portName;
      }) ?? [];

    if (yEdgeId && yEdge) {
      const currentEdge = reassembleEdge(yEdge);
      const updatedEdge = {
        ...currentEdge,
        [edgeType === "source" ? "sourceHandle" : "targetHandle"]:
          newPseudoPort.portName,
      };

      const newYEdge = yEdgeConstructor(updatedEdge);

      parentYEdges.set(yEdgeId, newYEdge);
    }
  } catch (error) {
    console.error("Error updating edges:", error);
  }

  return updatedPseudoPorts;
}

export function removeEdgePort(
  nodeId: string,
  parentYWorkflow: YWorkflow,
  portName: string,
  type: "source" | "target",
) {
  const parentYEdges = parentYWorkflow?.get("edges") as YEdgesMap | undefined;
  if (!parentYEdges) return;

  try {
    const edgesArray = Array.from(parentYEdges).map(([, e]) =>
      reassembleEdge(e as YEdge),
    );

    edgesArray.forEach((edgeObj) => {
      if (
        (type === "source" && edgeObj.source === nodeId) ||
        (type === "target" && edgeObj.target === nodeId)
      ) {
        if (type === "source" && edgeObj.sourceHandle === portName) {
          parentYEdges.delete(edgeObj.id);
        } else if (type === "target" && edgeObj.targetHandle === portName) {
          parentYEdges.delete(edgeObj.id);
        }
      }
    });
  } catch (error) {
    console.error("Error cleaning up edges:", error);
  }
}
