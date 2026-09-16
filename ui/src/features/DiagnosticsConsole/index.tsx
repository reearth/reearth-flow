import { DiagnosticsTable } from "@flow/components";
import useJobDiagnostics from "@flow/hooks/useJobDiagnostics";
import { useT } from "@flow/lib/i18n";

type Props = {
  jobId: string;
  /**
   * Whether the job is still running. Diagnostics are absent from the
   * jobStatus subscription payload, so they can only be kept current by
   * polling while the run is live.
   */
  isJobActive?: boolean;
  /**
   * Read the diagnostics attributed to one node instead of the job-level
   * bucket. Omit for the job-level view.
   */
  nodeId?: string;
  /** Controls rendered beside the table's search input. */
  leadingActions?: React.ReactNode;
};

/**
 * Structured engine diagnostics for one job. See `useJobDiagnostics` for how
 * the two API sources are merged and what the schema cannot currently reach.
 */
const DiagnosticsConsole: React.FC<Props> = ({
  jobId,
  isJobActive,
  nodeId,
  leadingActions,
}) => {
  const t = useT();

  const { diagnostics, isFetching } = useJobDiagnostics(
    jobId,
    isJobActive,
    nodeId,
  );

  // No wrapper: LogsConsole renders its table directly, and this view swaps
  // with it in place, so an extra flex container here would shift the panel.
  return (
    <DiagnosticsTable
      diagnostics={diagnostics}
      isFetching={isFetching && !diagnostics.length}
      leadingActions={leadingActions}
      flush
      noResultsMessage={t(
        "No diagnostics reported for this run yet. Diagnostics appear while a run is in progress and are persisted once it finishes.",
      )}
    />
  );
};

export default DiagnosticsConsole;
