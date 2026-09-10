import { useMemo } from "react";

import { useJob } from "@flow/lib/gql/job";
import {
  type Diagnostic,
  compareDiagnosticSeverity,
  isBlockingSeverity,
  isFatalDiagnostic,
} from "@flow/types";

/**
 * A job's diagnostics, merged from the two sources the API exposes and sorted
 * worst-first.
 *
 * `Job.failedNodes` holds every fatal row for the job; `nodeDiagnostics`
 * returns one bucket, the job-level one by default. The two genuinely overlap —
 * `failedNodes` selects on disposition alone and ignores nodeId, so a fatal row
 * carrying no nodeId is in both — so fatal rows are dropped from the bucket
 * before merging. That is exact rather than a dedupe heuristic: `failedNodes`
 * already holds every fatal row, so nothing is lost.
 *
 * Note the schema has no job-wide query. `nodeDiagnostics` filters on an exact
 * nodeId match, so rows attributed to a node are only reachable by naming it —
 * a successful run whose nodes emitted warnings has none of them here yet.
 *
 * Shared so that a caller needing only the count or severity (to badge a
 * control) reads exactly what the table will render. Every caller hits the same
 * query keys, so this costs one request no matter how many use it.
 */
export default (jobId?: string, isJobActive?: boolean, nodeId?: string) => {
  const { useGetJob, useGetJobDiagnostics } = useJob();

  const { job } = useGetJob(jobId);
  const { diagnostics, isFetching } = useGetJobDiagnostics(
    jobId,
    isJobActive,
    nodeId,
  );

  const merged: Diagnostic[] = useMemo(
    () =>
      [
        ...(job?.failedNodes ?? []),
        ...(diagnostics ?? []).filter(
          (diagnostic) => !isFatalDiagnostic(diagnostic),
        ),
      ].sort(compareDiagnosticSeverity),
    [job?.failedNodes, diagnostics],
  );

  // Sorted worst-first, so the head carries the worst severity present. Used
  // for the badge's colour: severity is a display concern, which is the one
  // thing it is for — pass/fail decisions read effectiveDisposition instead.
  const worstSeverity = merged[0]?.severity;

  return {
    diagnostics: merged,
    isFetching,
    count: merged.length,
    worstSeverity,
    hasBlocking: isBlockingSeverity(worstSeverity),
  };
};
