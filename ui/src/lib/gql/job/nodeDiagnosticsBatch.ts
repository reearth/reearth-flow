import { print } from "graphql";
import type { GraphQLClient } from "graphql-request";

import { DiagnosticFragmentDoc } from "../__gen__/plugins/graphql-request";
import type { DiagnosticFragment } from "../__gen__/plugins/graphql-request";

/**
 * Reads every bucket of a job's diagnostics in one request.
 *
 * `Job.nodeDiagnostics(nodeId:)` filters the job's rows by an exact nodeId
 * match, and the schema exposes no job-wide field, so the only way to see all
 * of a run's diagnostics is to ask for each bucket by name. GraphQL cannot map
 * a list argument onto repeated calls, so the field is aliased once per id and
 * the document is built here rather than generated.
 *
 * It is one request on purpose. The server's DiagnosticLoader caches the job's
 * rows for the life of a request, so N aliases cost one load of Redis, Mongo
 * and the GCS artifact — where N separate requests would cost N.
 *
 * `failedNodes` rides along here rather than on the shared `Job` fragment. It
 * has its own per-job resolver, so carrying it on the fragment made the
 * workspace jobs list resolve — and transfer — a diagnostics payload for every
 * row it never rendered.
 *
 * Replace this with a plain generated query the moment the API grows a job-wide
 * `Job.diagnostics` field; the loader already has `GetJobDiagnostics` behind it.
 */

/** The empty id is the bucket for rows that belong to no node. */
export const JOB_LEVEL_NODE_ID = "";

// Printed from the generated fragment rather than hand-listed, so the selection
// here cannot drift from the one every other diagnostics query uses.
const DIAGNOSTIC_FRAGMENT = print(DiagnosticFragmentDoc);

/**
 * Aliases are positional (`b0`, `b1`, …) rather than derived from the node id:
 * ids are UUIDs, which start with a digit often enough that using them directly
 * would produce invalid GraphQL names.
 */
export const buildNodeDiagnosticsBatchQuery = (bucketCount: number): string => {
  const params = Array.from(
    { length: bucketCount },
    (_, i) => `$n${i}: String!`,
  ).join(", ");
  const fields = Array.from(
    { length: bucketCount },
    (_, i) => `    b${i}: nodeDiagnostics(nodeId: $n${i}) { ...Diagnostic }`,
  ).join("\n");

  return `query GetNodeDiagnosticsBatch($jobId: ID!${params ? ", " + params : ""}) {
  job(id: $jobId) {
    id
    failedNodes { ...Diagnostic }
${fields}
  }
}

${DIAGNOSTIC_FRAGMENT}`;
};

type BatchResponse = {
  job?:
    | (Record<string, DiagnosticFragment[] | null> & {
        id: string;
        failedNodes?: DiagnosticFragment[] | null;
      })
    | null;
};

export type NodeDiagnosticsBatch = {
  /** Every fatal row for the job, regardless of which node it belongs to. */
  failedNodes: DiagnosticFragment[];
  /** Rows from the requested buckets, flattened. */
  bucketRows: DiagnosticFragment[];
};

/**
 * Fetches `failedNodes` and the given buckets in one round trip. Duplicate ids
 * are collapsed first: the same bucket asked for twice would return the same
 * rows twice, and there is no id on a Diagnostic to dedupe by afterwards.
 */
export const fetchNodeDiagnosticsBatch = async (
  client: GraphQLClient,
  jobId: string,
  nodeIds: string[],
): Promise<NodeDiagnosticsBatch> => {
  const buckets = Array.from(new Set(nodeIds));

  const variables: Record<string, string> = { jobId };
  buckets.forEach((nodeId, i) => {
    variables[`n${i}`] = nodeId;
  });

  const data = await client.request<BatchResponse>(
    buildNodeDiagnosticsBatchQuery(buckets.length),
    variables,
  );

  return {
    failedNodes: data.job?.failedNodes ?? [],
    bucketRows: buckets.flatMap((_, i) => data.job?.[`b${i}`] ?? []),
  };
};
