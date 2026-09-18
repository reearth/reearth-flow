import { useCallback } from "react";

import { DiagnosticsTable } from "@flow/components";
import useJobDiagnostics from "@flow/hooks/useJobDiagnostics";
import { useT } from "@flow/lib/i18n";
import type { Diagnostic } from "@flow/types";

type Props = {
  jobId: string;
  /**
   * Whether the job is still running. Diagnostics are absent from the
   * jobStatus subscription payload, so they can only be kept current by
   * polling while the run is live.
   */
  isJobActive?: boolean;
  /**
   * Every node in the run's workflows. Diagnostics are bucketed per node and
   * the API has no job-wide query, so a node missing here shows nothing.
   */
  nodeIds?: string[];
  /** Controls rendered beside the table's search input. */
  leadingActions?: React.ReactNode;
  /**
   * Reveals the action a diagnostic came from. Rows carry a node id at most —
   * a failure before the DAG starts carries none — so rows without one stay
   * inert rather than navigating somewhere arbitrary.
   */
  onNodeNavigate?: (nodeId: string) => void;
};

/**
 * Structured engine diagnostics for one job. See `useJobDiagnostics` for how
 * the two API sources are merged and what the schema cannot currently reach.
 */
const DiagnosticsConsole: React.FC<Props> = ({
  jobId,
  isJobActive,
  nodeIds,
  leadingActions,
  onNodeNavigate,
}) => {
  const t = useT();

  const { diagnostics, isFetching } = useJobDiagnostics(
    jobId,
    isJobActive,
    nodeIds,
  );

  const handleRowDoubleClick = useCallback(
    (diagnostic: Diagnostic) => {
      if (!diagnostic.nodeId) return;
      onNodeNavigate?.(diagnostic.nodeId);
    },
    [onNodeNavigate],
  );

  // No wrapper: LogsConsole renders its table directly, and this view swaps
  // with it in place, so an extra flex container here would shift the panel.
  return (
    <DiagnosticsTable
      diagnostics={diagnostics}
      isFetching={isFetching && !diagnostics.length}
      leadingActions={leadingActions}
      flush
      onRowDoubleClick={onNodeNavigate ? handleRowDoubleClick : undefined}
      noResultsMessage={t(
        "No diagnostics reported for this run yet. Diagnostics appear while a run is in progress and are persisted once it finishes.",
      )}
    />
  );
};

export default DiagnosticsConsole;
