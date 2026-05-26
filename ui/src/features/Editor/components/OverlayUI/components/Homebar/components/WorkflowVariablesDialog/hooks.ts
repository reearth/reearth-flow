import { useState, useEffect, useCallback, useMemo, useRef } from "react";
import { useY } from "react-yjs";
import { Map as YMap } from "yjs";

import { useEditorContext } from "@flow/features/Editor/editorContext";
import { useToast } from "@flow/features/NotificationSystem/useToast";
import { useWorkflowVars } from "@flow/hooks";
import { useT } from "@flow/lib/i18n";
import {
  computeSessionChanges,
  WorkflowVarSession,
} from "@flow/lib/yjs/workflowVarSession";
import { AwarenessUser, WorkflowVariable, VarType } from "@flow/types";
import {
  generateUUID,
  getDefaultConfigForWorkflowVar,
  getDefaultValueForWorkflowVar,
} from "@flow/utils";

export default ({
  currentWorkflowVariables,
  users,
  projectId,
  onClose,
  onAdd,
  onChange,
  onDelete,
  onDeleteBatch,
  onBatchUpdate,
}: {
  currentWorkflowVariables?: WorkflowVariable[];
  users?: Record<string, AwarenessUser>;
  projectId?: string;
  onClose: () => void;
  onAdd: (workflowVariable: WorkflowVariable) => Promise<void>;
  onChange: (workflowVariable: WorkflowVariable) => Promise<void>;
  onDelete: (id: string) => Promise<void>;
  onDeleteBatch?: (ids: string[]) => Promise<void>;
  onBatchUpdate?: (input: {
    projectId: string;
    creates?: {
      name: string;
      defaultValue: any;
      config?: any;
      type: VarType;
      required: boolean;
      publicValue: boolean;
      index?: number;
    }[];
    updates?: {
      paramId: string;
      name: string;
      defaultValue: any;
      config?: any;
      type: VarType;
      required: boolean;
      publicValue: boolean;
    }[];
    deletes?: string[];
    reorders?: {
      paramId: string;
      newIndex: number;
    }[];
  }) => Promise<void>;
}) => {
  const t = useT();
  const { toast } = useToast();
  const { yDoc, workflowVarAwareness } = useEditorContext();

  const myClientId = String(yDoc?.clientID ?? "local");

  // Shared Yjs session — all users read from and write to the same map so every
  // change is immediately visible to everyone (analogous to params in Yjs nodes).
  const yVarSession = useMemo(
    () => yDoc?.getMap<any>("workflowVarSession"),
    [yDoc],
  );
  const rawSession = useY(
    yVarSession ?? new YMap(),
  ) as Partial<WorkflowVarSession>;

  // Derive display variables and original base from session; fall back to server
  // state until the session is first initialised.
  const sessionVars = useMemo<WorkflowVariable[]>(
    () => rawSession?.variables ?? currentWorkflowVariables ?? [],
    [rawSession?.variables, currentWorkflowVariables],
  );

  const sessionBase = useMemo<WorkflowVariable[]>(
    () => rawSession?.base ?? currentWorkflowVariables ?? [],
    [rawSession?.base, currentWorkflowVariables],
  );

  const [isSubmitting, setIsSubmitting] = useState(false);
  // Track which variable is open in the sub-dialog (local per-user — only that
  // user cares; other users see it via awareness).
  const [editingVariableId, setEditingVariableId] = useState<string | null>(
    null,
  );

  // editingVariable is always derived from the live Yjs session, so sub-dialog
  // content updates instantly when another user edits the same variable.
  const editingVariable = editingVariableId
    ? (sessionVars.find((v) => v.id === editingVariableId) ?? null)
    : null;

  const { getUserFacingName } = useWorkflowVars();

  // ── Session helpers ────────────────────────────────────────────────────────

  // TODO(CRDT safety): for true per-item CRDT merging, migrate yVarSession to a
  // Y.Array of Y.Maps so concurrent edits to different variables do not resolve
  // as last-writer-wins on the whole array. For now, wrap multi-field writes in
  // a single Yjs transaction so they land atomically.
  const writeVars = useCallback(
    (newVars: WorkflowVariable[]) => {
      if (!yVarSession || !yDoc) return;
      yDoc.transact(() => {
        yVarSession.set("variables", newVars);
        yVarSession.set("timestamp", Date.now());
      });
    },
    [yVarSession, yDoc],
  );

  const clearSession = useCallback(() => {
    if (!yVarSession) return;
    yVarSession.delete("variables");
    yVarSession.delete("base");
    yVarSession.delete("timestamp");
    yVarSession.delete("pendingRefetch");
  }, [yVarSession]);

  // Initialise the shared session once from server data (first open wins).
  // Subsequent openers read the already-live session so they see each other's
  // in-progress edits immediately. If pendingRefetch is set, we join the
  // existing session and let the second useEffect below handle temp_ ID
  // resolution once TQ has returned fresh data — we never reinitialise here
  // from potentially stale currentWorkflowVariables.
  const hasInitRef = useRef(false);
  useEffect(() => {
    if (!yVarSession || currentWorkflowVariables === undefined) return;
    if (hasInitRef.current) return;
    hasInitRef.current = true;

    if (yVarSession.get("variables") === undefined) {
      // Fresh session — nobody has started editing yet.
      yVarSession.doc?.transact(() => {
        yVarSession.set("variables", [...currentWorkflowVariables]);
        yVarSession.set("base", [...currentWorkflowVariables]);
        yVarSession.set("timestamp", Date.now());
      });
    }
    // else: live session (possibly with pendingRefetch) — join without
    // overwriting. The effect below resolves temp_ IDs when TQ is ready.
  }, [yVarSession, currentWorkflowVariables]);

  // After a successful save with creates, resolve temp_ IDs once TQ has the
  // real server IDs. We use a per-mount "already processed" flag rather than
  // object-identity change detection so this fires correctly whether TQ was
  // already fresh at mount time or catches up later. Freshness is confirmed by
  // checking that currentWorkflowVariables is at least as long as the session
  // base (the saving client wrote base to include the temp_ variables).
  const hasProcessedPendingRefetchRef = useRef(false);
  useEffect(() => {
    if (!yVarSession || currentWorkflowVariables === undefined) return;

    if (!rawSession?.pendingRefetch) {
      hasProcessedPendingRefetchRef.current = false;
      return;
    }

    if (hasProcessedPendingRefetchRef.current) return;

    const sessionBaseLen = (
      (yVarSession.get("base") ?? []) as WorkflowVariable[]
    ).length;
    if (currentWorkflowVariables.length < sessionBaseLen) return; // TQ still stale

    hasProcessedPendingRefetchRef.current = true;
    yVarSession.doc?.transact(() => {
      yVarSession.set("variables", [...currentWorkflowVariables]);
      yVarSession.set("base", [...currentWorkflowVariables]);
      yVarSession.set("timestamp", Date.now());
      yVarSession.delete("pendingRefetch");
    });
  }, [yVarSession, currentWorkflowVariables, rawSession?.pendingRefetch]);

  // ── Always-current refs ───────────────────────────────────────────────────
  // All mutable state that cancel / unmount cleanup needs to read freshly.
  // Reading via ref inside a useCallback (or unmount closure) avoids stale
  // captures without adding those values to useCallback dep arrays.

  const sessionVarsRef = useRef(sessionVars);
  sessionVarsRef.current = sessionVars;

  const sessionBaseRef = useRef(sessionBase);
  sessionBaseRef.current = sessionBase;

  const usersRef = useRef(users);
  usersRef.current = users;

  const pendingRefetchRef = useRef<unknown>(rawSession?.pendingRefetch);
  pendingRefetchRef.current = rawSession?.pendingRefetch;

  const clearSessionRef = useRef(clearSession);
  clearSessionRef.current = clearSession;

  // ── Unsaved-changes detection ─────────────────────────────────────────────

  const hasUnsavedChanges = useMemo(() => {
    if (sessionVars.length !== sessionBase.length) return true;
    return sessionVars.some((v, i) => {
      const b = sessionBase[i];
      return !b || v.id !== b.id || JSON.stringify(v) !== JSON.stringify(b);
    });
  }, [sessionVars, sessionBase]);

  // ── Variable list mutations (all write directly to shared Yjs) ────────────

  // Snapshot of session vars at the moment this user joined (or initialised)
  // the dialog. Used in handleCancel to know what to revert TO for vars this
  // user changed. Captured once on the first non-null rawSession.variables.
  const joinedSessionVarsRef = useRef<WorkflowVariable[] | null>(null);
  useEffect(() => {
    if (joinedSessionVarsRef.current !== null) return;
    if (!rawSession?.variables) return;
    joinedSessionVarsRef.current = [
      ...(rawSession.variables as WorkflowVariable[]),
    ];
  }, [rawSession?.variables]);

  // Tracks temp_ variables this user added and their latest self-authored
  // state. Used in handleCancel to determine whether another collaborator has
  // modified an unconfirmed addition — if nobody else touched it, we remove
  // it when this user cancels; if someone did, we keep it.
  const myAddedTempVarsRef = useRef(new Map<string, WorkflowVariable>());

  // Tracks existing (non-temp_) variable IDs this user has modified. Used in
  // handleCancel to revert only this user's changes when others are present.
  const myChangedVarIdsRef = useRef(new Set<string>());

  // Tracks IDs this user explicitly deleted, split by kind so the cancel
  // revert can treat them differently:
  //  • real (non-temp_): need to be re-inserted at their original position
  //  • temp_: need to be removed even when another user renamed them
  const myDeletedRealVarIdsRef = useRef(new Set<string>());
  const myDeletedTempVarIdsRef = useRef(new Set<string>());

  const handleLocalAdd = useCallback(
    (type: VarType) => {
      const tempId = `temp_${generateUUID()}`;
      const newVariable: WorkflowVariable = {
        id: tempId,
        name: t("New Workflow Variable"),
        defaultValue: getDefaultValueForWorkflowVar(type),
        config: getDefaultConfigForWorkflowVar(type),
        type,
        required: true,
        public: true,
      };
      myAddedTempVarsRef.current.set(tempId, newVariable);
      writeVars([...sessionVarsRef.current, newVariable]);
    },
    [writeVars, t],
  );

  const handleUpdate = useCallback(
    (updatedVariable: WorkflowVariable) => {
      if (!updatedVariable.id.startsWith("temp_")) {
        myChangedVarIdsRef.current.add(updatedVariable.id);
      }
      if (myAddedTempVarsRef.current.has(updatedVariable.id)) {
        myAddedTempVarsRef.current.set(updatedVariable.id, updatedVariable);
      }
      writeVars(
        sessionVarsRef.current.map((v) =>
          v.id === updatedVariable.id ? updatedVariable : v,
        ),
      );
    },
    [writeVars],
  );

  const handleDeleteSingle = useCallback(
    (variableId: string) => {
      if (variableId.startsWith("temp_")) {
        myDeletedTempVarIdsRef.current.add(variableId);
      } else {
        myDeletedRealVarIdsRef.current.add(variableId);
      }
      writeVars(sessionVarsRef.current.filter((v) => v.id !== variableId));
    },
    [writeVars],
  );

  const handleReorder = useCallback(
    (oldIndex: number, newIndex: number) => {
      if (oldIndex === newIndex) return;
      const next = [...sessionVarsRef.current];
      const [moved] = next.splice(oldIndex, 1);
      next.splice(newIndex, 0, moved);
      writeVars(next);
    },
    [writeVars],
  );

  // ── Cancel session cleanup ────────────────────────────────────────────────
  // Extracted into a ref-based function so it can be called both from
  // handleCancel (explicit user action) and from the unmount cleanup
  // (navigation away without Cancel/Save). Reading all mutable state via refs
  // means the unmount closure never captures stale values.

  const performCancelCleanupRef = useRef<() => void>(() => {});
  performCancelCleanupRef.current = () => {
    const currentUsers = usersRef.current;
    const otherUsersInDialog = Object.values(currentUsers ?? {}).some(
      (u) =>
        u.openWorkflowVariablesDialog && String(u.clientId) !== myClientId,
    );

    if (!otherUsersInDialog) {
      if (pendingRefetchRef.current && yVarSession) {
        // A save with creates is pending temp_→real ID resolution. Wiping the
        // session here would delete the pendingRefetch flag, so the next opener
        // would init from a potentially stale TQ cache and miss the new variable.
        // Instead, revert variables back to the saved base (discarding this
        // user's unsaved additions) while keeping pendingRefetch so the next
        // opener's watcher can resolve temp_ IDs once TQ is fresh.
        yVarSession.doc?.transact(() => {
          yVarSession.set("variables", [...sessionBaseRef.current]);
          yVarSession.set("timestamp", Date.now());
        });
      } else {
        // No pending save — wipe the session entirely.
        clearSessionRef.current();
      }
    } else {
      // Other collaborators are still editing.
      // 1. Revert existing vars this user changed back to their joined-state values.
      // 2. Remove temp_ vars this user added that nobody else has touched.
      // 3. Restore real vars this user deleted (re-insert at original position).
      const myChangedIds = myChangedVarIdsRef.current;
      const myTempIds = myAddedTempVarsRef.current;
      const myDeletedRealIds = myDeletedRealVarIdsRef.current;
      const myDeletedTempIds = myDeletedTempVarIdsRef.current;
      const joinedVars = joinedSessionVarsRef.current;
      const joinedMap = joinedVars
        ? new Map(joinedVars.map((v) => [v.id, v]))
        : null;

      const currentVars = sessionVarsRef.current;

      const reverted = currentVars
        .map((v) => {
          if (v.id.startsWith("temp_")) return v;
          if (myChangedIds.has(v.id) && joinedMap?.has(v.id)) {
            return joinedMap.get(v.id) as WorkflowVariable;
          }
          return v;
        })
        .filter((v) => {
          if (!v.id.startsWith("temp_") || !myTempIds.has(v.id)) return true;
          // If this user explicitly deleted it, always remove it regardless of
          // whether another user modified the name in the meantime.
          if (myDeletedTempIds.has(v.id)) return false;
          // Remove if nobody else modified it from our last-authored version.
          return JSON.stringify(v) !== JSON.stringify(myTempIds.get(v.id));
        });

      // Re-insert real vars this user deleted at their original joined position.
      if (joinedVars && myDeletedRealIds.size > 0) {
        for (let i = 0; i < joinedVars.length; i++) {
          const jv = joinedVars[i];
          if (!myDeletedRealIds.has(jv.id)) continue;
          if (reverted.some((v) => v.id === jv.id)) continue; // already present
          // Find insertion point: after the nearest preceding joined var that is
          // still in reverted, so relative order is preserved.
          let insertAt = reverted.length;
          for (let j = i - 1; j >= 0; j--) {
            const prevIdx = reverted.findIndex(
              (v) => v.id === joinedVars[j].id,
            );
            if (prevIdx !== -1) {
              insertAt = prevIdx + 1;
              break;
            }
          }
          reverted.splice(insertAt, 0, jv);
        }
      }

      const changed =
        reverted.length !== currentVars.length ||
        reverted.some(
          (v, i) => JSON.stringify(v) !== JSON.stringify(currentVars[i]),
        );

      if (changed && yVarSession) {
        yVarSession.doc?.transact(() => {
          yVarSession.set("variables", reverted);
          yVarSession.set("timestamp", Date.now());
        });
      }
    }
  };

  // ── Submit / Cancel ───────────────────────────────────────────────────────

  // Set to true by handleSubmit and handleCancel so the unmount cleanup knows
  // not to duplicate the session revert (the dialog is closing intentionally).
  const hasExplicitlyClosedRef = useRef(false);

  const handleSubmit = useCallback(async () => {
    setIsSubmitting(true);
    try {
      const changes = computeSessionChanges(
        sessionVarsRef.current,
        sessionBaseRef.current,
      );
      const hasChanges =
        changes.creates.length > 0 ||
        changes.updates.length > 0 ||
        changes.deletes.length > 0 ||
        changes.reorders.length > 0;

      if (onBatchUpdate && projectId && hasChanges) {
        await onBatchUpdate({
          projectId,
          ...(changes.creates.length > 0 && { creates: changes.creates }),
          ...(changes.updates.length > 0 && { updates: changes.updates }),
          ...(changes.deletes.length > 0 && { deletes: changes.deletes }),
          ...(changes.reorders.length > 0 && { reorders: changes.reorders }),
        });
      } else if (hasChanges) {
        // Fallback: individual API calls (no reorders supported here)
        for (const c of changes.creates) {
          const variable = sessionVarsRef.current.find(
            (v) => v.id.startsWith("temp_") && v.name === c.name,
          );
          if (variable) await onAdd(variable);
        }
        for (const u of changes.updates) {
          const variable = sessionVarsRef.current.find(
            (v) => v.id === u.paramId,
          );
          if (variable) await onChange(variable);
        }
        if (changes.deletes.length > 0) {
          if (onDeleteBatch && changes.deletes.length > 1) {
            await onDeleteBatch(changes.deletes);
          } else {
            for (const id of changes.deletes) {
              await onDelete(id);
            }
          }
        }
      }

      await new Promise((resolve) => setTimeout(resolve, 100));

      // Write the committed state to Yjs so users who still have the dialog
      // open see the saved data immediately (instead of the stale server
      // fallback that clearSession would have caused).
      // If creates were submitted, mark pendingRefetch so the session is
      // re-initialised with real server IDs once TanStack Query refetches.
      if (yVarSession) {
        yVarSession.doc?.transact(() => {
          yVarSession.set("variables", [...sessionVarsRef.current]);
          yVarSession.set("base", [...sessionVarsRef.current]);
          yVarSession.set("timestamp", Date.now());
          if (hasChanges && changes.creates.length > 0) {
            yVarSession.set("pendingRefetch", myClientId);
          }
        });
      }

      hasExplicitlyClosedRef.current = true;
      workflowVarAwareness?.onDialogClose();
      onClose();
    } catch (error) {
      console.error("Failed to submit workflow variable changes:", error);
    } finally {
      setIsSubmitting(false);
    }
  }, [
    onBatchUpdate,
    projectId,
    onAdd,
    onChange,
    onDeleteBatch,
    onDelete,
    yVarSession,
    myClientId,
    workflowVarAwareness,
    onClose,
  ]);

  const handleCancel = useCallback(() => {
    performCancelCleanupRef.current();
    hasExplicitlyClosedRef.current = true;
    workflowVarAwareness?.onDialogClose();
    onClose();
  }, [workflowVarAwareness, onClose]);

  // ── Awareness lifecycle ───────────────────────────────────────────────────

  // Broadcast awareness that this user has the dialog open. On unmount, clear
  // awareness and — if the dialog was not explicitly closed via Cancel/Save
  // (e.g., the user navigated away) — revert/clear the Yjs session so stale
  // temp_ variables do not leak into the next open.
  const workflowVarAwarenessRef = useRef(workflowVarAwareness);
  workflowVarAwarenessRef.current = workflowVarAwareness;
  useEffect(() => {
    workflowVarAwarenessRef.current?.onDialogOpen();
    return () => {
      workflowVarAwarenessRef.current?.onDialogClose();
      if (!hasExplicitlyClosedRef.current) {
        performCancelCleanupRef.current();
      }
    };
  }, []);

  // ── VariableEditDialog open/close ─────────────────────────────────────────

  const handleEditVariable = useCallback(
    (variable: WorkflowVariable) => {
      setEditingVariableId(variable.id);
      workflowVarAwareness?.onEditStart(variable.id);
    },
    [workflowVarAwareness],
  );

  const handleCloseEdit = useCallback(() => {
    setEditingVariableId(null);
    workflowVarAwareness?.onEditStart(null);
    // No broadcastDraft needed — the shared Yjs session already holds the
    // latest state written by handleUpdate.
  }, [workflowVarAwareness]);

  // If a collaborator deletes the variable currently open in the sub-dialog,
  // close it explicitly and notify this user rather than silently wiping their
  // unsaved edits via the null-variable path in VariableEditDialog's useEffect.
  useEffect(() => {
    if (!editingVariableId) return;
    if (sessionVars.some((v) => v.id === editingVariableId)) return;
    handleCloseEdit();
    toast({
      title: t("Variable removed"),
      description: t(
        "Another collaborator deleted the variable you were editing.",
      ),
    });
  }, [sessionVars, editingVariableId, handleCloseEdit, toast, t]);

  // ── Public interface ──────────────────────────────────────────────────────

  // Expose as workflowVariables (renamed from localWorkflowVariables) so
  // callers know it comes from shared Yjs state.
  return {
    workflowVariables: sessionVars,
    hasUnsavedChanges,
    isSubmitting,
    editingVariable,
    myClientId,
    getUserFacingName,
    handleLocalAdd,
    handleUpdate,
    handleDeleteSingle,
    handleReorder,
    handleSubmit,
    handleCancel,
    handleEditVariable,
    handleCloseEdit,
  };
};
