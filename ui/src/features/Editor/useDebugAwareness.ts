import { useCallback, useEffect, useState } from "react";
import type { Awareness } from "y-protocols/awareness";

import { UserDebug } from "@flow/types";

export default ({ yAwareness }: { yAwareness: Awareness }) => {
  const [activeDebugRuns, setActiveDebugRuns] = useState<UserDebug[]>([]);

  const broadcastDebugRun = useCallback(
    (jobId: string | null) => {
      if (jobId) {
        yAwareness.setLocalStateField("debugRun", {
          jobId,
          startedAt: Date.now(),
        });
      } else {
        const state = yAwareness.getLocalState();
        if (state?.debugRun) {
          const { debugRun, ...rest } = state;
          yAwareness.setLocalState(rest);
        }
      }
    },
    [yAwareness],
  );

  useEffect(() => {
    const handleChange = () => {
      const myClientId = yAwareness.clientID;
      const states = Array.from(yAwareness.getStates());
      // Find all other users with active debug runs
      const otherDebugRuns = states
        .filter(
          ([clientId, state]) =>
            state.debugRun && state.debugRun.jobId && clientId !== myClientId,
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
  }, [yAwareness]);

  return {
    broadcastDebugRun,
    activeDebugRuns,
  };
};
