import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import type { Job } from "@flow/types";
import {
  OrderDirection,
  type PaginationOptions,
} from "@flow/types/paginationOptions";
import { isDefined } from "@flow/utils";

import { CancelJobInput } from "../__gen__/graphql";
import { toDiagnostic, toJob } from "../convert";
import { useGraphQLContext } from "../provider";

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

/**
 * `nodeDiagnostics` filters the job's rows by an exact nodeId match, and rows
 * that belong to the job rather than to any one node carry no nodeId. An empty
 * string is therefore the job-level bucket, not "all nodes".
 */
export const JOB_LEVEL_NODE_ID = "";

export const useQueries = () => {
  const graphQLContext = useGraphQLContext();
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
   * One bucket of a job's diagnostics. `nodeDiagnostics` filters the job's rows
   * by an exact nodeId match, so the default empty id returns the job-level
   * rows and a real node id returns that node's.
   *
   * There is deliberately no job-wide query on the schema, so this cannot stand
   * in for one. The terminal per-node failures come from `Job.failedNodes`
   * instead, which the Job fragment already carries.
   */
  const useGetJobDiagnosticsQuery = (
    jobId?: string,
    poll?: boolean,
    nodeId: string = JOB_LEVEL_NODE_ID,
  ) =>
    useQuery({
      queryKey: [JobQueryKeys.GetJobDiagnostics, jobId, nodeId],
      queryFn: async () => {
        const data = await graphQLContext?.GetJobDiagnostics({
          jobId: jobId ?? "",
          nodeId,
        });
        // An empty list is a legitimate answer, not a failure: live rows come
        // from a TTL-bound cache that is only merged with the persisted rows at
        // job completion, so there is a window right after a job starts where
        // nothing exists yet.
        return (data?.job?.nodeDiagnostics ?? []).map(toDiagnostic);
      },
      enabled: !!jobId,
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
