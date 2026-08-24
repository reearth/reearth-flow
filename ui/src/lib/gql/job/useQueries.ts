import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import type { Job } from "@flow/types";
import {
  OrderDirection,
  type PaginationOptions,
} from "@flow/types/paginationOptions";
import { isDefined } from "@flow/utils";

import { CancelJobInput } from "../__gen__/graphql";
import { toJob, toNodeExecution } from "../convert";
import { useGraphQLContext } from "../provider";

export enum JobQueryKeys {
  GetJobs = "getJobs",
  GetJob = "getJob",
  GetNodeExecutions = "getNodeExecutions",
}

export const JOBS_FETCH_RATE = 15;

/**
 * Diagnostics and feature counts are not part of the nodeStatus/jobStatus
 * subscription payloads — those carry the status enum and nothing else — so the
 * only way to show them while a job runs is to poll for them.
 */
export const NODE_EXECUTIONS_POLL_RATE = 5000;

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

  const useGetNodeExecutionsQuery = (jobId?: string, poll?: boolean) =>
    useQuery({
      queryKey: [JobQueryKeys.GetNodeExecutions, jobId],
      queryFn: async () => {
        const data = await graphQLContext?.GetNodeExecutions({
          jobId: jobId ?? "",
        });
        // An empty list is a legitimate answer, not a failure: live rows come
        // from a TTL-bound cache that is only merged with the persisted rows at
        // job completion, so there is a window right after a job starts where
        // nothing exists yet.
        return (data?.nodeExecutions ?? []).map(toNodeExecution);
      },
      enabled: !!jobId,
      refetchInterval: poll ? NODE_EXECUTIONS_POLL_RATE : false,
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
    useGetNodeExecutionsQuery,
    cancelJobMutation,
  };
};
