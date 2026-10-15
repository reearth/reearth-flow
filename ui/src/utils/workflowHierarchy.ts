import { DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";
import type { Workflow } from "@flow/types";

/** Maps each subworkflow id to the id of the workflow whose node references it. */
export type WorkflowParentMap = Record<string, string>;

export const buildWorkflowParentMap = (
  rawWorkflows: Workflow[],
): WorkflowParentMap => {
  const parentMap: WorkflowParentMap = {};
  for (const workflow of rawWorkflows) {
    for (const node of workflow.nodes ?? []) {
      const subworkflowId = node.data?.subworkflowId;
      if (subworkflowId && subworkflowId !== workflow.id) {
        parentMap[subworkflowId] = workflow.id;
      }
    }
  }
  return parentMap;
};

/**
 * The chain of workflows from the main workflow down to `workflowId`,
 * both ends included. A workflow nothing references hangs off the main one.
 */
export const getWorkflowLineage = (
  parentMap: WorkflowParentMap,
  workflowId: string,
): string[] => {
  const lineage = [workflowId];
  const seen = new Set(lineage);
  let current = workflowId;
  while (current !== DEFAULT_ENTRY_GRAPH_ID) {
    const parent = parentMap[current] ?? DEFAULT_ENTRY_GRAPH_ID;
    // Guards against a malformed project whose subworkflows reference each other.
    if (seen.has(parent)) break;
    lineage.unshift(parent);
    seen.add(parent);
    current = parent;
  }
  return lineage;
};

export const isWorkflowDescendant = (
  parentMap: WorkflowParentMap,
  workflowId: string,
  ancestorId: string,
): boolean =>
  workflowId !== ancestorId &&
  getWorkflowLineage(parentMap, workflowId).includes(ancestorId);
