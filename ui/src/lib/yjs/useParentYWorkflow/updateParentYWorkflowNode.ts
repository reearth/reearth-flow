import { Node, PseudoPort } from "@flow/types";

import { yNodeConstructor } from "../conversions";
import type { YNodesArray } from "../types";

import { getUpdatedPseudoPortsParam } from "./utils";

export function updateParentYWorkflowNode(
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

  updatePseudoPorts(
    parentNodes,
    parentYNodes,
    subworkflowParentNode,
    prevNode,
    newPseudoPort,
  );
}

function updatePseudoPorts(
  parentNodes: Node[],
  parentYNodes: YNodesArray,
  subworkflowParentNode: Node,
  prevNode: Node,
  newPseudoPort: PseudoPort,
) {
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
