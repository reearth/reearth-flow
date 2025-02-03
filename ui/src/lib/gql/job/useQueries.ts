import { useInfiniteQuery, useQuery } from "@tanstack/react-query";

import type { Job } from "@flow/types";
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

  const useGetJobsInfiniteQuery = (workspaceId?: string) =>
    useInfiniteQuery({
      queryKey: [JobQueryKeys.GetJobs, workspaceId],
      initialPageParam: 1,
      queryFn: async ({ pageParam }) => {
        const data = await graphQLContext?.GetJobs({
          workspaceId: workspaceId ?? "",
          pagination: {
            page: pageParam,
            pageSize: JOBS_FETCH_RATE,
            // orderDir: "ASC",
          },
        });
        if (!data) return;
        const {
          jobsPage: {
            nodes,
            pageInfo: { totalCount, currentPage, totalPages },
          },
        } = data;
        const jobs: Job[] = nodes.filter(isDefined).map((job) => toJob(job));
        return { jobs, totalCount, currentPage, totalPages };
      },
      enabled: !!workspaceId,
      getNextPageParam: (lastPage) => {
        if (!lastPage) return undefined;
        if ((lastPage.currentPage ?? 0) < (lastPage.totalPages ?? 0)) {
          return (lastPage.currentPage ?? 0) + 1;
        }
        return undefined;
      },
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
    useGetJobsInfiniteQuery,
    useGetJobQuery,
  };
};
