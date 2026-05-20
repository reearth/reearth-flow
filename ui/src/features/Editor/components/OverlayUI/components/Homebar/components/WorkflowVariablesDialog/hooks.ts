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
import { WorkflowVariable, VarType } from "@flow/types";
import {
  generateUUID,
  getDefaultConfigForWorkflowVar,
  getDefaultValueForWorkflowVar,
} from "@flow/utils";

export default ({
  currentWorkflowVariables,
  projectId,
  onClose,
  onAdd,
  onChange,
  onDelete,
  onDeleteBatch,
  onBatchUpdate,
}: {
  currentWorkflowVariables?: WorkflowVariable[];
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
  // in-progress edits immediately.
  const hasInitRef = useRef(false);
  useEffect(() => {
    if (!yVarSession || currentWorkflowVariables === undefined) return;
    if (hasInitRef.current) return;
    hasInitRef.current = true;

    if (yVarSession.get("pendingRefetch") !== undefined) {
      // A previous save created new variables and then the saving client left.
      // Reinit from current server data (which now has real IDs).
      yVarSession.doc?.transact(() => {
        yVarSession.set("variables", [...currentWorkflowVariables]);
        yVarSession.set("base", [...currentWorkflowVariables]);
        yVarSession.set("timestamp", Date.now());
        yVarSession.delete("pendingRefetch");
      });
    } else if (yVarSession.get("variables") === undefined) {
      // Fresh session — nobody has started editing yet.
      yVarSession.doc?.transact(() => {
        yVarSession.set("variables", [...currentWorkflowVariables]);
        yVarSession.set("base", [...currentWorkflowVariables]);
        yVarSession.set("timestamp", Date.now());
      });
    }
    // else: live session already in progress — join without overwriting.
  }, [yVarSession, currentWorkflowVariables]);

  // After a successful save, wait for TanStack Query to refetch and then
  // reinitialise the session with real server IDs (resolving any temp_ IDs).
  // Any client still in the dialog can do the reinit — all write the same
  // server-authoritative data, so last-writer-wins is safe.
  const prevWorkflowVarsRef = useRef(currentWorkflowVariables);
  useEffect(() => {
    if (!yVarSession || currentWorkflowVariables === undefined) return;

    if (!rawSession?.pendingRefetch) {
      prevWorkflowVarsRef.current = currentWorkflowVariables;
      return;
    }

    // Wait until currentWorkflowVariables has actually changed (i.e. the
    // server has returned fresh data after the save).
    if (prevWorkflowVarsRef.current === currentWorkflowVariables) return;

    yVarSession.doc?.transact(() => {
      yVarSession.set("variables", [...currentWorkflowVariables]);
      yVarSession.set("base", [...currentWorkflowVariables]);
      yVarSession.set("timestamp", Date.now());
      yVarSession.delete("pendingRefetch");
    });
    prevWorkflowVarsRef.current = currentWorkflowVariables;
  }, [yVarSession, currentWorkflowVariables, rawSession?.pendingRefetch]);

  // Broadcast awareness that this user has the dialog open, and clean up on
  // unmount (navigation / accidental close without Cancel/Save).
  const workflowVarAwarenessRef = useRef(workflowVarAwareness);
  workflowVarAwarenessRef.current = workflowVarAwareness;
  useEffect(() => {
    workflowVarAwarenessRef.current?.onDialogOpen();
    return () => {
      workflowVarAwarenessRef.current?.onDialogClose();
    };
  }, []);

  // ── Unsaved-changes detection ─────────────────────────────────────────────

  const hasUnsavedChanges = useMemo(() => {
    if (sessionVars.length !== sessionBase.length) return true;
    return sessionVars.some((v, i) => {
      const b = sessionBase[i];
      return !b || v.id !== b.id || JSON.stringify(v) !== JSON.stringify(b);
    });
  }, [sessionVars, sessionBase]);

  // ── Variable list mutations (all write directly to shared Yjs) ────────────

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
      writeVars([...sessionVars, newVariable]);
    },
    [writeVars, sessionVars, t],
  );

  const handleUpdate = useCallback(
    (updatedVariable: WorkflowVariable) => {
      writeVars(
        sessionVars.map((v) =>
          v.id === updatedVariable.id ? updatedVariable : v,
        ),
      );
    },
    [writeVars, sessionVars],
  );

  const handleDeleteSingle = useCallback(
    (variableId: string) => {
      writeVars(sessionVars.filter((v) => v.id !== variableId));
    },
    [writeVars, sessionVars],
  );

  const handleReorder = useCallback(
    (oldIndex: number, newIndex: number) => {
      if (oldIndex === newIndex) return;
      const next = [...sessionVars];
      const [moved] = next.splice(oldIndex, 1);
      next.splice(newIndex, 0, moved);
      writeVars(next);
    },
    [writeVars, sessionVars],
  );

  // ── Submit / Cancel ───────────────────────────────────────────────────────

  const handleSubmit = useCallback(async () => {
    setIsSubmitting(true);
    try {
      const changes = computeSessionChanges(sessionVars, sessionBase);
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
          const variable = sessionVars.find(
            (v) => v.id.startsWith("temp_") && v.name === c.name,
          );
          if (variable) await onAdd(variable);
        }
        for (const u of changes.updates) {
          const variable = sessionVars.find((v) => v.id === u.paramId);
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
          yVarSession.set("variables", [...sessionVars]);
          yVarSession.set("base", [...sessionVars]);
          yVarSession.set("timestamp", Date.now());
          if (hasChanges && changes.creates.length > 0) {
            yVarSession.set("pendingRefetch", myClientId);
          }
        });
      }

      workflowVarAwareness?.onDialogClose();
      onClose();
    } catch (error) {
      console.error("Failed to submit workflow variable changes:", error);
    } finally {
      setIsSubmitting(false);
    }
  }, [
    sessionVars,
    sessionBase,
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
    clearSession();
    workflowVarAwareness?.onDialogClose();
    onClose();
  }, [clearSession, workflowVarAwareness, onClose]);

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
