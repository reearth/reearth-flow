import { DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";
import { Node, Workflow } from "@flow/types";

export function computeWorkflowPath(
  rawWorkflows: Workflow[],
  currentWorkflowId?: string,
): string {
  const pathParts: string[] = [];

  let workflowId = currentWorkflowId;
  while (workflowId) {
    if (workflowId === DEFAULT_ENTRY_GRAPH_ID) break;
    pathParts.unshift(workflowId);

    const parentWorkflow = rawWorkflows.find((w) => {
      const nodes = w.nodes as Node[];
      return nodes.some((n) => n.data.subworkflowId === workflowId);
    });

    workflowId = parentWorkflow?.id;
  }

  return pathParts.join(".");
}
