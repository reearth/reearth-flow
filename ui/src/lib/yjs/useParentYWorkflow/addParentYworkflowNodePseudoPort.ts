import * as Y from "yjs";

import { DEFAULT_ROUTING_PORT } from "@flow/global-constants";
import type { Node, Workflow } from "@flow/types";

import { yNodeConstructor } from "../conversions";
import { YNodesMap, YWorkflow } from "../types";

function findParentWorkflowWithSubworkflowNode(
  currentWorkflowId: string,
  rawWorkflows: Workflow[],
  yWorkflows: Y.Map<YWorkflow>,
) {
  const parentWorkflow = rawWorkflows.find((w) => {
    const nodes = w.nodes as Node[];
    return nodes.some((n) => n.data.subworkflowId === currentWorkflowId);
  });
  if (!parentWorkflow) return undefined;
  const parentYWorkflow = yWorkflows.get(parentWorkflow.id);
  if (!parentYWorkflow) return undefined;
  const parentYNodes = parentYWorkflow.get("nodes") as YNodesMap;
  const parentNodes = Object.values(parentYNodes.toJSON()) as Node[];
  const subworkflowNode = parentNodes.find(
    (n) => n.data.subworkflowId === currentWorkflowId,
  );

  return { subworkflowNode, parentYNodes };
}

// Add a pseudoInput or pseudoOutput to the parent subworkflow node if the new node is an InputRouter or OutputRouter and the parent subworkflow node doesn't already have one.
export function addParentYWorkflowNodePseudoPort(
  newNode: Node,
  rawWorkflows: Workflow[],
  yWorkflows: Y.Map<YWorkflow>,
  currentYWorkflow?: YWorkflow,
) {
  const isInputRouter = newNode.data.officialName === "InputRouter";
  const isOutputRouter = newNode.data.officialName === "OutputRouter";
  let shouldCreatePseudoPort = false;
  const currentWorkflowId = currentYWorkflow?.get("id")?.toJSON() as string;
  const parentWorkflowInfo = findParentWorkflowWithSubworkflowNode(
    currentWorkflowId,
    rawWorkflows,
    yWorkflows,
  );

  if (parentWorkflowInfo?.subworkflowNode) {
    shouldCreatePseudoPort =
      (isInputRouter &&
        !parentWorkflowInfo.subworkflowNode.data.pseudoInputs?.length) ||
      (isOutputRouter &&
        !parentWorkflowInfo.subworkflowNode.data.pseudoOutputs?.length);
  }

  if (shouldCreatePseudoPort) {
    newNode.data.params = {
      ...newNode.data.params,
      routingPort: DEFAULT_ROUTING_PORT,
    };

    if (parentWorkflowInfo?.subworkflowNode) {
      const newPseudoPort = {
        nodeId: newNode.id,
        portName: DEFAULT_ROUTING_PORT,
      };

      const updatedSubworkflowNode = { ...parentWorkflowInfo.subworkflowNode };

      if (isInputRouter) {
        updatedSubworkflowNode.data.pseudoInputs = [newPseudoPort];
      } else if (isOutputRouter) {
        updatedSubworkflowNode.data.pseudoOutputs = [newPseudoPort];
      }

      parentWorkflowInfo.parentYNodes.set(
        parentWorkflowInfo.subworkflowNode.id,
        yNodeConstructor(updatedSubworkflowNode),
      );
    }
  }
}
