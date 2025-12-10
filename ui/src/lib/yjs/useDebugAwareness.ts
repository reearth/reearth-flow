import { useCallback, useEffect, useRef, useState } from "react";
import type { Awareness } from "y-protocols/awareness";

import { AwarenessUser, UserDebugRun } from "@flow/types";

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

  // Track last known debugRun per client (after project filter)
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

      // Fresh data directly from awareness
      const states = yAwareness.getStates() as Map<number, AwarenessUser>;
      const debugRunsByClient = prevDebugRunsByClientRef.current;

      const changedClients = [...added, ...updated, ...removed];
      let hasDebugRunChange = false;

      // Only inspect clients that actually changed
      for (const clientId of changedClients) {
        if (clientId === myClientId) continue;

        const state = states.get(clientId) as
          | (AwarenessUser & { debugRun?: UserDebugRun })
          | undefined;

        const rawDebugRun = state?.debugRun;

        // Apply project filter & normalize to null if we don't care about it
        const currentDebugRun =
          rawDebugRun &&
          rawDebugRun.jobId &&
          (!projectId || rawDebugRun.projectId === projectId)
            ? rawDebugRun
            : null;

        const prevDebugRun = debugRunsByClient.get(clientId) ?? null;

        const same =
          !!currentDebugRun === !!prevDebugRun &&
          (!currentDebugRun ||
            (currentDebugRun.jobId === prevDebugRun.jobId &&
              currentDebugRun.status === prevDebugRun.status &&
              currentDebugRun.startedAt === prevDebugRun.startedAt &&
              currentDebugRun.projectId === prevDebugRun.projectId));

        if (!same) {
          hasDebugRunChange = true;

          if (currentDebugRun) {
            debugRunsByClient.set(clientId, currentDebugRun);
          } else {
            debugRunsByClient.delete(clientId);
          }
        }
      }

      // Nothing about debugRun changed for any relevant client → bail
      if (!hasDebugRunChange) {
        return;
      }

      // Rebuild list of users with active debug runs from our cache
      const nextActiveUsers: AwarenessUser[] = [];

      for (const [clientId, debugRun] of debugRunsByClient.entries()) {
        if (clientId === myClientId) continue;

        const state = states.get(clientId) as AwarenessUser | undefined;
        if (!state) continue;

        // Ensure the debugRun we expose matches the cached one
        nextActiveUsers.push({
          ...(state as AwarenessUser),
          debugRun,
        });
      }

      setActiveUsersDebugRuns(nextActiveUsers);
    };

    // Initial call to populate state – treat all current clients as "added"
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
