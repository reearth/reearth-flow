import { Node, PseudoPort } from "@flow/types";

import type { YEdgesMap, YNodesMap, YWorkflow } from "../types";

import { updateParentYWorkflowEdges } from "./updateParentYWorkflowEdges";
import { updateParentYWorkflowNode } from "./updateParentYWorkflowNode";

export function updateParentYWorkflow(
  currentWorkflowId: string,
  parentYWorkflow: YWorkflow,
  prevNode: Node,
  newParams: any,
) {
  const parentYNodes = parentYWorkflow.get("nodes") as YNodesMap;
  const parentYEdges = parentYWorkflow.get("edges") as YEdgesMap;

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
