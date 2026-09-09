import { useMemo } from "react";

import { useIndexedDB } from "@flow/lib/indexedDB";
import { useCurrentProject } from "@flow/stores";
import { compareDiagnosticSeverity } from "@flow/types";

import useJobDiagnostics from "./useJobDiagnostics";

/**
 * Worst diagnostic severity per node for the current project's debug run, so
 * the canvas can mark the actions that reported something.
 *
 * Deliberately does not poll. It shares query keys with the debug panel, so
 * while that panel is open and polling a live run these markers follow along
 * for free; on their own they would put the whole canvas on a timer.
 *
 * Diagnostics with no nodeId are skipped — they cannot be attributed to an
 * action. That is not rare: a failure before the DAG starts (a misconfigured
 * required parameter, say) carries no node context at all, so the panel's badge
 * is the only place those surface.
 */
export default () => {
  const [currentProject] = useCurrentProject();
  const { value: debugRunState } = useIndexedDB("debugRun");

  const jobId = useMemo(
    () =>
      debugRunState?.jobs?.find((job) => job.projectId === currentProject?.id)
        ?.jobId,
    [debugRunState, currentProject?.id],
  );

  const { diagnostics } = useJobDiagnostics(jobId);

  return useMemo(() => {
    const byNode = new Map<string, string>();
    // Sorted worst-first already, so the first row seen for a node wins and
    // later, milder rows for the same node must not overwrite it.
    for (const diagnostic of [...diagnostics].sort(compareDiagnosticSeverity)) {
      if (!diagnostic.nodeId || byNode.has(diagnostic.nodeId)) continue;
      byNode.set(diagnostic.nodeId, diagnostic.severity);
    }
    return byNode;
  }, [diagnostics]);
};
