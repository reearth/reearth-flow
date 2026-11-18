import { useCallback, useEffect, useRef, useState } from "react";
import type { Awareness } from "y-protocols/awareness";

import type { WorkflowVariable } from "@flow/types";

import type { WorkflowVarSession } from "./workflowVarSession";

/** The awareness field each client in the dialog publishes its copy under. */
export const WORKFLOW_VAR_SESSION_FIELD = "workflowVarSession";

const isSession = (value: unknown): value is WorkflowVarSession =>
  typeof value === "object" &&
  value !== null &&
  Array.isArray((value as WorkflowVarSession).variables) &&
  Array.isArray((value as WorkflowVarSession).base) &&
  typeof (value as WorkflowVarSession).rev === "number";

/**
 * The shared session as every client sees it: the copy with the highest
 * revision, ties going to the higher client id so that all clients agree.
 */
export const readWorkflowVarSession = (
  awareness: Awareness,
): WorkflowVarSession | undefined => {
  let best: WorkflowVarSession | undefined;
  let bestClientId = -1;
  awareness.getStates().forEach((state, clientId) => {
    const session: unknown = state?.[WORKFLOW_VAR_SESSION_FIELD];
    if (!isSession(session)) return;
    if (
      !best ||
      session.rev > best.rev ||
      (session.rev === best.rev && clientId > bestClientId)
    ) {
      best = session;
      bestClientId = clientId;
    }
  });
  return best;
};

const readOwnSession = (awareness: Awareness) => {
  const session: unknown =
    awareness.getLocalState()?.[WORKFLOW_VAR_SESSION_FIELD];
  return isSession(session) ? session : undefined;
};

/**
 * The Workflow Variables dialog's shared, unsaved editing session.
 *
 * Variables are saved through the API, so the session is not part of the
 * workflow document and never touches it: writing it there would make opening
 * or cancelling the dialog an edit of the workflow. It lives in awareness
 * instead. Every client with the dialog open keeps a copy of the newest
 * revision, so the session outlives any one of them leaving, and it is gone
 * once the last one leaves. Nothing is published until someone edits.
 */
export default function useWorkflowVarSession(
  awareness: Awareness | null | undefined,
) {
  const [session, setSession] = useState<WorkflowVarSession | undefined>(() =>
    awareness ? readWorkflowVarSession(awareness) : undefined,
  );
  // Once this client has left, it must not take a copy back from the others.
  const hasLeftRef = useRef(false);

  useEffect(() => {
    if (!awareness) return;

    const handleChange = () => {
      const latest = readWorkflowVarSession(awareness);
      // Keep a copy of a newer revision, synchronously: the client that
      // published it may be about to leave and take its copy with it.
      const own = readOwnSession(awareness);
      if (!hasLeftRef.current && latest && (!own || latest.rev > own.rev)) {
        awareness.setLocalStateField(WORKFLOW_VAR_SESSION_FIELD, latest);
      }
      setSession(latest);
    };

    handleChange();
    awareness.on("change", handleChange);
    return () => awareness.off("change", handleChange);
  }, [awareness]);

  /** Publishes a new revision of the session. */
  const publishSession = useCallback(
    (next: {
      variables: WorkflowVariable[];
      base: WorkflowVariable[];
      pendingRefetch?: boolean;
    }) => {
      if (!awareness) return;
      hasLeftRef.current = false;
      const rev = (readWorkflowVarSession(awareness)?.rev ?? 0) + 1;
      const session: WorkflowVarSession = { ...next, rev };
      awareness.setLocalStateField(WORKFLOW_VAR_SESSION_FIELD, session);
    },
    [awareness],
  );

  /** Drops this client's copy, as it leaves the dialog. */
  const leaveSession = useCallback(() => {
    hasLeftRef.current = true;
    awareness?.setLocalStateField(WORKFLOW_VAR_SESSION_FIELD, null);
  }, [awareness]);

  /** The session as it stands now, for reads outside render. */
  const getSession = useCallback(
    () => (awareness ? readWorkflowVarSession(awareness) : undefined),
    [awareness],
  );

  return { session, getSession, publishSession, leaveSession };
}
