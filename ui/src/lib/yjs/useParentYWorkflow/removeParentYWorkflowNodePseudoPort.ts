import { Node } from "@flow/types";

import { yNodeConstructor } from "../conversions";
import { YNodesArray, YWorkflow } from "../types";

import { removeEdgePort } from "./updateParentYWorkflowEdges";
import { splitPorts } from "./utils";

export function removeParentYWorkflowNodePseudoPort(
  currentWorkflowId: string,
  parentYWorkflow: YWorkflow,
  nodeToDelete: Node,
) {
  const parentYNodes = parentYWorkflow?.get("nodes") as YNodesArray | undefined;
  if (!parentYNodes) return;

  const parentNodes = parentYNodes.toJSON() as Node[];

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

  const newParentNodes = parentNodes.map((node) =>
    node.id === subworkflowNode.id ? updatedSubworkflowNode : node,
  );

  const newParentYNodes = newParentNodes.map((node) => yNodeConstructor(node));
  parentYNodes.delete(0, parentNodes.length);
  parentYNodes.insert(0, newParentYNodes);
}
