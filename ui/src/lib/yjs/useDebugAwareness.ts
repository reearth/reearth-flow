import { useCallback, useEffect, useRef, useState } from "react";
import type { Awareness } from "y-protocols/awareness";

import { AwarenessUser } from "@flow/types";

export default ({
  yAwareness,
  projectId,
}: {
  yAwareness: Awareness;
  projectId?: string;
}) => {
  const [activeUsersDebugRuns, setActiveUsersDebugRuns] = useState<
    AwarenessUser[]
  >([]);
  const prevDebugRunsRef = useRef<string>("");
  const prevDebugRunsByClientRef = useRef<Map<number, any>>(new Map());
  const broadcastDebugRun = useCallback(
    (jobId: string | null, status?: string) => {
      if (jobId && projectId) {
        yAwareness.setLocalStateField("debugRun", {
          jobId,
          projectId,
          startedAt: Date.now(),
          status,
        });
      } else {
        const state = yAwareness.getLocalState();
        if (state?.debugRun) {
          yAwareness.setLocalStateField("debugRun", null);
        }
      }
    },
    [yAwareness, projectId],
  );
  useEffect(() => {
    const handleChange = ({
      added,
      updated,
      removed,
    }: {
      added: number[];
      updated: number[];
      removed: number[];
    }) => {
      const myClientId = yAwareness.clientID;
      // Get fresh data directly from awareness, not from useUsers
      const states = yAwareness.getStates();

      // Check if any of the changed clients have debugRun modifications
      const changedClients = [...added, ...updated, ...removed];
      let hasDebugRunChange = false;

      for (const clientId of changedClients) {
        if (clientId === myClientId) continue;

        const state = states.get(clientId) as Record<string, any> | undefined;
        const currentDebugRun = state?.debugRun;
        const prevDebugRun = prevDebugRunsByClientRef.current.get(clientId);

        // Check if debugRun actually changed (not just cursor movement)
        const debugRunChanged =
          JSON.stringify(currentDebugRun) !== JSON.stringify(prevDebugRun);

        if (debugRunChanged) {
          hasDebugRunChange = true;
          break;
        }
      }

      // Early exit if no debugRun-related changes
      if (!hasDebugRunChange) {
        return;
      }

      // Now create snapshot to check if actual debugRun data changed
      const debugRunsSnapshot = Array.from(
        states.entries() as IterableIterator<[number, AwarenessUser]>,
      )
        .filter(
          ([clientId, state]: [number, Record<string, any>]) =>
            clientId !== myClientId &&
            state.debugRun &&
            state.debugRun.jobId &&
            (!projectId || state.debugRun.projectId === projectId),
        )
        .map(([_clientId, state]: [number, Record<string, any>]) => [
          state.debugRun.jobId,
          state.debugRun.status,
          state.debugRun.startedAt,
        ]);

      const currentSnapshot = JSON.stringify(debugRunsSnapshot);

      // Early exit if debugRun data hasn't actually changed
      if (currentSnapshot === prevDebugRunsRef.current) {
        return;
      }
      prevDebugRunsRef.current = currentSnapshot;

      // Update the per-client debugRun tracking
      prevDebugRunsByClientRef.current.clear();
      for (const [clientId, state] of states.entries()) {
        const stateObj = state as Record<string, any>;
        if (stateObj.debugRun) {
          prevDebugRunsByClientRef.current.set(clientId, stateObj.debugRun);
        }
      }

      // Now do the full mapping with all user info
      const otherDebugRuns = Array.from(
        states.entries() as IterableIterator<[number, AwarenessUser]>,
      )
        .filter(
          ([clientId, state]) =>
            state.debugRun &&
            state.debugRun.jobId &&
            clientId !== myClientId &&
            (!projectId || state.debugRun.projectId === projectId),
        )
        .map(([_clientId, state]) => state as AwarenessUser);
      setActiveUsersDebugRuns(otherDebugRuns);
    };

    // Initial call to populate state - manually call with empty change set
    const initialStates = yAwareness.getStates();
    const allClientIds = Array.from(initialStates.keys());
    handleChange({ added: allClientIds, updated: [], removed: [] });

    yAwareness.on("change", handleChange);

    return () => {
      yAwareness.off("change", handleChange);
    };
  }, [yAwareness, projectId]);

  // Cleanup: Clear broadcast when component unmounts
  useEffect(() => {
    return () => {
      // Clear debug run from awareness directly to avoid dependency issues
      const state = yAwareness.getLocalState();
      if (state?.debugRun) {
        yAwareness.setLocalStateField("debugRun", null);
      }
    };
    // Only run on unmount, not when yAwareness changes
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return {
    activeUsersDebugRuns,
    broadcastDebugRun,
  };
};
