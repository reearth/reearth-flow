import { Node } from "@flow/types";

import { yNodeConstructor } from "../conversions";
import { YNodesMap, YWorkflow } from "../types";

import { removeEdgePort } from "./updateParentYWorkflowEdges";
import { splitPorts } from "./utils";

export function removeParentYWorkflowNodePseudoPort(
  currentWorkflowId: string,
  parentYWorkflow: YWorkflow,
  nodeToDelete: Node,
) {
  const parentYNodes = parentYWorkflow?.get("nodes") as YNodesMap | undefined;
  if (!parentYNodes) return;

  const parentNodes = Object.values(parentYNodes.toJSON()) as Node[];

  const subworkflowNode = parentNodes.find(
    (n) => n.data.subworkflowId === currentWorkflowId,
  );
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
        updatedSubworkflowNode.id,
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
        updatedSubworkflowNode.id,
        parentYWorkflow,
        portToRemove.portName,
        "source",
      );
    }
  }

  parentYNodes.set(
    subworkflowNode.id,
    yNodeConstructor(updatedSubworkflowNode),
  );
}
