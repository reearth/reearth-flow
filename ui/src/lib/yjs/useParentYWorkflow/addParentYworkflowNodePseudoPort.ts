import * as Y from "yjs";

import { DEFAULT_ROUTING_PORT } from "@flow/global-constants";
import type { Node, Workflow } from "@flow/types";

import { yNodeConstructor } from "../conversions";
import { YNodesMap, YWorkflow } from "../types";

function findParentWorkflow(
  currentWorkflowId: string,
  rawWorkflows: Workflow[],
) {
  const parentWorkflow = rawWorkflows.find((w) => {
    const nodes = w.nodes as Node[];
    return nodes.some((n) => n.data.subworkflowId === currentWorkflowId);
  });

  return parentWorkflow;
}

export function addParentYWorkflowNodePseudoPort(
  newNode: Node,
  rawWorkflows: Workflow[],
  yWorkflows: Y.Map<YWorkflow>,
  currentWorkflowId: string,
) {
  const isInputRouter = newNode.data.officialName === "InputRouter";
  const isOutputRouter = newNode.data.officialName === "OutputRouter";
  let hasNoPseudoInputsOrOutputs = false;
  if (isInputRouter || isOutputRouter) {
    const parentWorkflow = findParentWorkflow(currentWorkflowId, rawWorkflows);

    if (parentWorkflow) {
      const parentYWorkflow = yWorkflows.get(parentWorkflow.id);
      if (parentYWorkflow) {
        const parentYNodes = parentYWorkflow.get("nodes") as YNodesMap;
        const parentNodes = Object.values(parentYNodes.toJSON()) as Node[];
        const subworkflowNode = parentNodes.find(
          (n) => n.data.subworkflowId === currentWorkflowId,
        );

        if (subworkflowNode) {
          hasNoPseudoInputsOrOutputs =
            (isInputRouter && !subworkflowNode.data.pseudoInputs?.length) ||
            (isOutputRouter && !subworkflowNode.data.pseudoOutputs?.length);
        }
      }
    }
  }

  if (hasNoPseudoInputsOrOutputs) {
    newNode.data.params = {
      ...newNode.data.params,
      routingPort: DEFAULT_ROUTING_PORT,
    };
  }

  if (hasNoPseudoInputsOrOutputs) {
    const parentWorkflow = findParentWorkflow(currentWorkflowId, rawWorkflows);

    if (parentWorkflow) {
      const parentYWorkflow = yWorkflows.get(parentWorkflow.id);
      if (parentYWorkflow) {
        const parentYNodes = parentYWorkflow.get("nodes") as YNodesMap;
        const parentNodes = Object.values(parentYNodes.toJSON()) as Node[];
        const subworkflowNode = parentNodes.find(
          (n) => n.data.subworkflowId === currentWorkflowId,
        );

        if (subworkflowNode) {
          const newPseudoPort = {
            nodeId: newNode.id,
            portName: DEFAULT_ROUTING_PORT,
          };

          const updatedSubworkflowNode = { ...subworkflowNode };

          if (isInputRouter) {
            updatedSubworkflowNode.data.pseudoInputs = [newPseudoPort];
          } else if (isOutputRouter) {
            updatedSubworkflowNode.data.pseudoOutputs = [newPseudoPort];
          }

          parentYNodes.set(
            subworkflowNode.id,
            yNodeConstructor(updatedSubworkflowNode),
          );
        }
      }
    }
  }
}
