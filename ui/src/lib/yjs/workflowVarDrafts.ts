import { WorkflowVariable } from "@flow/types";

export type WorkflowVarDraft = {
  variables: WorkflowVariable[];
  timestamp: number;
  editingVariableId: string | null;
};

export type WorkflowVarDraftStore = Record<string, WorkflowVarDraft>;

/**
 * Returns the most recent draft from any client other than the current user.
 * Last-write-wins on the full variable list so add/delete/reorder/field updates
 * are all captured in one broadcast.
 */
export function getMostRecentOtherDraft(
  myClientId: string,
  rawDrafts: WorkflowVarDraftStore,
): WorkflowVarDraft | null {
  let best: WorkflowVarDraft | null = null;

  for (const [clientId, draft] of Object.entries(rawDrafts)) {
    if (clientId === myClientId) continue;
    if (!best || draft.timestamp > best.timestamp) {
      best = draft;
    }
  }

  return best;
}
