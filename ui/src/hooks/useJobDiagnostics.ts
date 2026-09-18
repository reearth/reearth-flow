import { useEffect, useMemo, useRef } from "react";

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
 * `failedNodes` is the terminal failure list and the buckets are everything
 * else, but they overlap once a job finishes: `failedNodes` selects on
 * disposition alone and ignores nodeId, so a fatal row can be in both.
 *
 * Overlapping rows are matched rather than filtered wholesale. `GetFailedNodes`
 * reads only Mongo and the completion artifact — never Redis — so while a job
 * runs it is empty and every fatal row lives in the live buckets. Dropping all
 * fatal bucket rows therefore hid live failures from the table and the badge
 * until the run ended, which is when watching matters least.
 *
 * The schema has no job-wide query — `nodeDiagnostics` filters on an exact
 * nodeId match — so `nodeIds` must list every node in the run's workflows. A
 * node missing from it contributes no rows and nothing says so, which is
 * exactly how a successful run full of dropped features once looked empty.
 *
 * Shared so that a caller needing only the count or severity (to badge a
 * control) reads exactly what the table will render. Every caller hits the same
 * query key, so this costs one request no matter how many use it.
 */
/**
 * Mirrors the server's `dedupeDiagnostics`, which keys on
 * (nodeID, code, disposition) and normalises an absent value to "". Using the
 * same key means the only rows collapsed here are ones the server would have
 * collapsed itself. The NUL separator stops a value containing the delimiter
 * from forging a collision.
 */
const diagnosticKey = (diagnostic: Diagnostic) =>
  [
    diagnostic.nodeId ?? "",
    diagnostic.code,
    diagnostic.effectiveDisposition ?? "",
  ].join("\u0000");

export default (jobId?: string, isJobActive?: boolean, nodeIds?: string[]) => {
  const { useGetJobDiagnostics } = useJob();

  const { failedNodes, bucketRows, isFetching, refetch } = useGetJobDiagnostics(
    jobId,
    isJobActive,
    nodeIds,
  );

  // Polling stops the moment a run ends, but `failedNodes` is written at
  // completion — so the last poll of a failing run saw everything except the
  // terminal rows. Without this read the panel can sit open on a stale count
  // until it remounts.
  const wasActiveRef = useRef(false);
  useEffect(() => {
    if (isJobActive) {
      wasActiveRef.current = true;
      return;
    }
    if (!wasActiveRef.current) return;
    wasActiveRef.current = false;
    refetch();
  }, [isJobActive, refetch]);

  const terminalKeys = useMemo(
    () => new Set((failedNodes ?? []).map(diagnosticKey)),
    [failedNodes],
  );

  const merged: Diagnostic[] = useMemo(
    () =>
      [
        ...(failedNodes ?? []),
        ...(bucketRows ?? []).filter(
          // failedNodes only ever holds fatal rows, so a non-fatal bucket row
          // cannot be a duplicate and need not be looked up at all.
          (diagnostic) =>
            !isFatalDiagnostic(diagnostic) ||
            !terminalKeys.has(diagnosticKey(diagnostic)),
        ),
      ].sort(compareDiagnosticSeverity),
    [failedNodes, bucketRows, terminalKeys],
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
