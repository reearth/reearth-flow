import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import type { Job } from "@flow/types";
import { OrderDirection } from "@flow/types/paginationOptions";
import type { PaginationOptions } from "@flow/types/paginationOptions";
import { isDefined } from "@flow/utils";

import type { CancelJobInput } from "../__gen__/graphql";
import { toDiagnostic, toJob } from "../convert";
import { useGraphQLClient, useGraphQLContext } from "../provider";

import {
  JOB_LEVEL_NODE_ID,
  fetchNodeDiagnosticsBatch,
} from "./nodeDiagnosticsBatch";

export enum JobQueryKeys {
  GetJobs = "getJobs",
  GetJob = "getJob",
  GetJobDiagnostics = "getJobDiagnostics",
}

export const JOBS_FETCH_RATE = 15;

/**
 * Diagnostics are not part of the jobStatus subscription payload — it carries
 * the status enum and nothing else — so the only way to show them accumulate
 * while a job runs is to poll for them.
 */
export const JOB_DIAGNOSTICS_POLL_RATE = 5000;

export const useQueries = () => {
  const graphQLContext = useGraphQLContext();
  const graphQLClient = useGraphQLClient();
  const queryClient = useQueryClient();

  const useGetJobsQuery = (
    workspaceId?: string,
    keyword?: string,
    paginationOptions?: PaginationOptions,
  ) =>
    useQuery({
      queryKey: [JobQueryKeys.GetJobs, workspaceId],
      queryFn: async () => {
        const data = await graphQLContext?.GetJobs({
          workspaceId: workspaceId ?? "",
          keyword,
          pagination: {
            page: paginationOptions?.page ?? 1,
            pageSize: JOBS_FETCH_RATE,
            orderDir: paginationOptions?.orderDir ?? OrderDirection.Desc,
            orderBy: paginationOptions?.orderBy ?? "startedAt",
          },
        });
        if (!data) return;
        const {
          jobs: {
            nodes,
            pageInfo: { totalCount, currentPage, totalPages },
          },
        } = data;

        const jobs: Job[] = nodes.filter(isDefined).map((job) => toJob(job));
        return { jobs, totalCount, currentPage, totalPages };
      },
      enabled: !!workspaceId,
    });

  const useGetJobQuery = (jobId?: string) =>
    useQuery({
      queryKey: [JobQueryKeys.GetJob, jobId],
      queryFn: async () => {
        const data = await graphQLContext?.GetJob({ id: jobId ?? "" });
        if (!data?.job) return;
        return toJob(data.job);
      },
      enabled: !!jobId,
    });

  /**
   * Every bucket of a job's diagnostics: the job-level rows plus one per node.
   *
   * `nodeDiagnostics` filters by an exact nodeId match and the schema has no
   * job-wide field, so the buckets have to be named. They go in a single
   * aliased request — see `nodeDiagnosticsBatch` for why that is one server-side
   * load rather than N.
   *
   * `nodeIds` should be every node in the run's workflows. Omitting a node
   * silently hides its diagnostics, which is the failure mode this replaced.
   *
   * `failedNodes` comes back from the same request rather than from the shared
   * Job fragment, which would make the jobs list resolve it per row.
   */
  const useGetJobDiagnosticsQuery = (
    jobId?: string,
    poll?: boolean,
    nodeIds?: string[],
  ) =>
    useQuery({
      queryKey: [JobQueryKeys.GetJobDiagnostics, jobId, nodeIds],
      queryFn: async () => {
        if (!graphQLClient || !jobId)
          return { failedNodes: [], bucketRows: [] };
        const { failedNodes, bucketRows } = await fetchNodeDiagnosticsBatch(
          graphQLClient,
          jobId,
          [JOB_LEVEL_NODE_ID, ...(nodeIds ?? [])],
        );
        // Empty is a legitimate answer, not a failure: live rows come from a
        // TTL-bound cache that is only merged with the persisted rows at job
        // completion, so there is a window right after a job starts where
        // nothing exists yet.
        return {
          failedNodes: failedNodes.map(toDiagnostic),
          bucketRows: bucketRows.map(toDiagnostic),
        };
      },
      enabled: !!jobId && !!graphQLClient,
      refetchInterval: poll ? JOB_DIAGNOSTICS_POLL_RATE : false,
    });

  const cancelJobMutation = useMutation({
    mutationFn: async ({ jobId }: { jobId: string }) => {
      const input: CancelJobInput = {
        jobId,
      };

      const data = await graphQLContext?.CancelJob({
        input,
      });

      if (data?.cancelJob.job) {
        return toJob(data.cancelJob.job);
      }
    },
    onSuccess: (job) => {
      // TODO: Maybe update cache and not refetch? What happens after pagination?
      queryClient.invalidateQueries({
        queryKey: [JobQueryKeys.GetJobs, job?.workspaceId],
      });
    },
  });

  return {
    useGetJobsQuery,
    useGetJobQuery,
    useGetJobDiagnosticsQuery,
    cancelJobMutation,
  };
};
