import { useMemo } from "react";

import { DiagnosticsTable } from "@flow/components";
import { useJob } from "@flow/lib/gql/job";
import { useT } from "@flow/lib/i18n";
import { compareDiagnosticSeverity, isFatalDiagnostic } from "@flow/types";

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
};

/**
 * Structured engine diagnostics for one job.
 *
 * Shows what the API can express: `Job.failedNodes` — every fatal row for the
 * job — together with one bucket of `nodeDiagnostics`.
 *
 * The two sources genuinely overlap, so the bucket is filtered before being
 * merged. `failedNodes` selects on disposition alone and ignores nodeId, so a
 * fatal row that carries no nodeId appears in both it and the job-level bucket.
 * Dropping fatal rows from the bucket is exact rather than a dedupe heuristic:
 * `failedNodes` already holds every fatal row, so nothing is lost.
 *
 * The schema has no job-wide diagnostics query: `nodeDiagnostics` filters by an
 * exact nodeId match, so diagnostics attributed to a specific node are only
 * reachable by naming that node. A run that only produced per-node warnings
 * therefore shows nothing here until a `nodeId` is passed.
 */
const DiagnosticsConsole: React.FC<Props> = ({
  jobId,
  isJobActive,
  nodeId,
}) => {
  const t = useT();

  const { useGetJob, useGetJobDiagnostics } = useJob();

  const { job } = useGetJob(jobId);
  const { diagnostics, isFetching } = useGetJobDiagnostics(
    jobId,
    isJobActive,
    nodeId,
  );

  const sorted = useMemo(
    () =>
      [
        ...(job?.failedNodes ?? []),
        ...(diagnostics ?? []).filter(
          (diagnostic) => !isFatalDiagnostic(diagnostic),
        ),
      ].sort(compareDiagnosticSeverity),
    [job?.failedNodes, diagnostics],
  );

  return (
    <div className="flex h-full min-h-0 flex-col overflow-auto">
      <DiagnosticsTable
        diagnostics={sorted}
        isFetching={isFetching && !sorted.length}
        noResultsMessage={t(
          "No diagnostics reported for this run yet. Diagnostics appear while a run is in progress and are persisted once it finishes.",
        )}
      />
    </div>
  );
};

export default DiagnosticsConsole;
