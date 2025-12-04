import { useCallback, useEffect, useState } from "react";
import type { Awareness } from "y-protocols/awareness";

import { UserDebug } from "@flow/types";

export default ({
  yAwareness,
  projectId,
}: {
  yAwareness: Awareness;
  projectId?: string;
}) => {
  const [activeDebugRuns, setActiveDebugRuns] = useState<UserDebug[]>([]);

  const broadcastDebugRun = useCallback(
    (jobId: string | null) => {
      if (jobId && projectId) {
        yAwareness.setLocalStateField("debugRun", {
          jobId,
          projectId,
          startedAt: Date.now(),
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
    const handleChange = () => {
      const myClientId = yAwareness.clientID;
      const states = Array.from(yAwareness.getStates());
      // Find all other users with active debug runs in this project
      const otherDebugRuns = states
        .filter(
          ([clientId, state]) =>
            state.debugRun &&
            state.debugRun.jobId &&
            clientId !== myClientId &&
            (!projectId || state.debugRun.projectId === projectId), // Filter by project
        )
        .map(([clientId, state]) => ({
          userId: String(clientId),
          userName: state.userName || "Unknown User",
          jobId: state.debugRun.jobId,
          startedAt: state.debugRun.startedAt || Date.now(),
        }));

      setActiveDebugRuns(otherDebugRuns);
    };

    // Initial call to populate state
    handleChange();

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
    broadcastDebugRun,
    activeDebugRuns,
  };
};
