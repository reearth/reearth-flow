import { useInfiniteQuery, useQuery } from "@tanstack/react-query";

import type { Job } from "@flow/types";
import { isDefined } from "@flow/utils";

import { toJob } from "../convert";
import { useGraphQLContext } from "../provider";

export enum JobQueryKeys {
  GetJobs = "getJobs",
  GetJob = "getJob",
}

export const useQueries = () => {
  const graphQLContext = useGraphQLContext();

  const useGetJobsInfiniteQuery = (workspaceId?: string, fetchRate?: number) =>
    useInfiniteQuery({
      queryKey: [JobQueryKeys.GetJobs, workspaceId],
      initialPageParam: null,
      queryFn: async ({ pageParam }) => {
        const data = await graphQLContext?.GetJobs({
          workspaceId: workspaceId ?? "",
          pagination: {
            first: fetchRate,
            after: pageParam,
          },
        });
        if (!data) return;
        const {
          jobs: {
            nodes,
            pageInfo: { endCursor, hasNextPage },
            totalCount,
          },
        } = data;
        const jobs: Job[] = nodes.filter(isDefined).map((job) => toJob(job));
        return { jobs, endCursor, hasNextPage, totalCount };
      },
      enabled: !!workspaceId,
      getNextPageParam: (lastPage) => {
        if (!lastPage) return undefined;
        const { endCursor, hasNextPage } = lastPage;
        return hasNextPage ? endCursor : undefined;
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
