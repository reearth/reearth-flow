import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import type { Job } from "@flow/types";
import {
  OrderDirection,
  type PaginationOptions,
} from "@flow/types/paginationOptions";
import { isDefined } from "@flow/utils";

import { CancelJobInput } from "../__gen__/graphql";
import { toEdgeExecution, toJob } from "../convert";
import { useGraphQLContext } from "../provider";

export enum JobQueryKeys {
  GetJobs = "getJobs",
  GetJob = "getJob",
  GetEdgeExecution = "getEdgeExecution",
}

export const JOBS_FETCH_RATE = 15;

export const useQueries = () => {
  const graphQLContext = useGraphQLContext();
  const queryClient = useQueryClient();

  const useGetJobsQuery = (
    workspaceId?: string,
    paginationOptions?: PaginationOptions,
  ) =>
    useQuery({
      queryKey: [JobQueryKeys.GetJobs, workspaceId],
      queryFn: async () => {
        const data = await graphQLContext?.GetJobs({
          workspaceId: workspaceId ?? "",
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

  const useGetEdgeExecutionQuery = (
    jobId?: string,
    edgeId?: string,
    disabled?: boolean,
  ) =>
    useQuery({
      queryKey: [JobQueryKeys.GetEdgeExecution, jobId, edgeId],
      queryFn: async () => {
        const data = await graphQLContext?.GetEdgeExecution({
          jobId: jobId ?? "",
          edgeId: edgeId ?? "",
        });
        if (!data?.edgeExecution) return;
        return toEdgeExecution(data.edgeExecution);
      },
      enabled: !disabled && !!jobId && !!edgeId,
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
    useGetEdgeExecutionQuery,
    cancelJobMutation,
  };
};
