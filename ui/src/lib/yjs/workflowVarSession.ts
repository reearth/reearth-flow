import type { WorkflowVariable, VarType } from "@flow/types";

/**
 * The Workflow Variables dialog's shared editing session, kept in awareness by
 * useWorkflowVarSession.
 */
export type WorkflowVarSession = {
  variables: WorkflowVariable[];
  /** The saved variables the edits are diffed against. */
  base: WorkflowVariable[];
  /** Revision: the highest wins. */
  rev: number;
  // Set after a save that created variables: the session still holds their
  // temp_ IDs, so it is replaced with server data once TanStack Query has
  // refetched them.
  pendingRefetch?: boolean;
};

export type SessionCreate = {
  name: string;
  defaultValue: any;
  config?: any;
  type: VarType;
  required: boolean;
  publicValue: boolean;
  index?: number;
};

export type SessionUpdate = {
  paramId: string;
  name: string;
  defaultValue: any;
  config?: any;
  type: VarType;
  required: boolean;
  publicValue: boolean;
};

export type SessionChanges = {
  creates: SessionCreate[];
  updates: SessionUpdate[];
  deletes: string[];
  reorders: { paramId: string; newIndex: number }[];
};

/**
 * Diffs the current session variable list against the original server base to
 * produce the set of creates/updates/deletes/reorders needed by the batch API.
 */
export function computeSessionChanges(
  current: WorkflowVariable[],
  base: WorkflowVariable[],
): SessionChanges {
  const baseIndexMap = new Map<string, number>(base.map((v, i) => [v.id, i]));
  const baseVarMap = new Map<string, WorkflowVariable>(
    base.map((v) => [v.id, v]),
  );
  const currentIdSet = new Set(current.map((v) => v.id));

  const creates: SessionCreate[] = current
    .map((v, i) => ({ v, i }))
    .filter(({ v }) => v.id.startsWith("temp_"))
    .map(({ v, i }) => ({
      name: v.name,
      defaultValue: v.defaultValue,
      config: v.config,
      type: v.type,
      required: v.required,
      publicValue: v.public,
      index: i,
    }));

  const updates: SessionUpdate[] = current
    .filter((v) => !v.id.startsWith("temp_") && baseVarMap.has(v.id))
    .filter((v) => {
      const b = baseVarMap.get(v.id);
      return b && JSON.stringify(v) !== JSON.stringify(b);
    })
    .map((v) => ({
      paramId: v.id,
      name: v.name,
      defaultValue: v.defaultValue,
      config: v.config,
      type: v.type,
      required: v.required,
      publicValue: v.public,
    }));

  const deletes: string[] = base
    .filter((v) => !currentIdSet.has(v.id))
    .map((v) => v.id);

  const reorders: { paramId: string; newIndex: number }[] = current
    .map((v, i) => ({ v, i }))
    .filter(({ v }) => !v.id.startsWith("temp_") && baseIndexMap.has(v.id))
    .filter(({ v, i }) => baseIndexMap.get(v.id) !== i)
    .map(({ v, i }) => ({ paramId: v.id, newIndex: i }));

  return { creates, updates, deletes, reorders };
}
