import { Node, PseudoPort } from "@flow/types";

import type { YEdgesArray, YNodesArray, YWorkflow } from "../types";

import { updateParentYWorkflowEdges } from "./updateParentYWorkflowEdges";
import { updateParentYWorkflowNode } from "./updateParentYWorkflowNode";

export function updateParentYWorkflow(
  currentWorkflowId: string,
  parentYWorkflow: YWorkflow,
  prevNode: Node,
  newParams: any,
) {
  const parentYNodes = parentYWorkflow.get("nodes") as YNodesArray;
  const parentYEdges = parentYWorkflow.get("edges") as YEdgesArray;

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
