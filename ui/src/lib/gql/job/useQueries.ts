import { useQuery } from "@tanstack/react-query";

import type { Job } from "@flow/types";
import type { PaginationOptions } from "@flow/types/paginationOptions";
import { isDefined } from "@flow/utils";

import { toJob } from "../convert";
import { useGraphQLContext } from "../provider";

export enum JobQueryKeys {
  GetJobs = "getJobs",
  GetJob = "getJob",
}

const JOBS_FETCH_RATE = 15;

export const useQueries = () => {
  const graphQLContext = useGraphQLContext();

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
            pageSize: paginationOptions?.pageSize ?? JOBS_FETCH_RATE,
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
  const useGetJobQuery = (jobId: string) =>
    useQuery({
      queryKey: [JobQueryKeys.GetJob, jobId],
      queryFn: async () => {
        const data = await graphQLContext?.GetJob({ id: jobId });
        if (!data?.job) return;
        return toJob(data.job);
      },
    });

  return {
    useGetJobsQuery,
    useGetJobQuery,
  };
};
