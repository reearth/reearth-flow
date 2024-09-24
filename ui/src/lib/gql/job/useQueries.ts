import { useInfiniteQuery, useQuery } from "@tanstack/react-query";

import type { Job, JobStatus } from "@flow/types";
import { isDefined } from "@flow/utils";

import { JobFragment, JobStatus as GraphqlJobStatus } from "../__gen__/graphql";
import { useGraphQLContext } from "../provider";

enum JobQueryKeys {
  GetJobs = "getJobs",
  GetJob = "getJob",
}

const JOBS_FETCH_RATE = 15;

export const useQueries = () => {
  const graphQLContext = useGraphQLContext();

  const useGetJobsInfiniteQuery = (workspaceId?: string) =>
    useInfiniteQuery({
      queryKey: [JobQueryKeys.GetJobs, workspaceId],
      initialPageParam: null,
      queryFn: async ({ pageParam }) => {
        const data = await graphQLContext?.GetJobs({
          workspaceId: workspaceId ?? "",
          pagination: {
            first: JOBS_FETCH_RATE,
            after: pageParam,
          },
        });
        if (!data) return;
        const {
          jobs: {
            nodes,
            pageInfo: { endCursor, hasNextPage },
          },
        } = data;
        const jobs: Job[] = nodes
          .filter(isDefined)
          .map((job) => createNewJobObject(job));
        return { jobs, endCursor, hasNextPage };
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
        return createNewJobObject(data.job);
      },
    });

  return {
    useGetJobsInfiniteQuery,
    useGetJobQuery,
  };
};

function toJobStatus(status: GraphqlJobStatus): JobStatus {
  switch (status) {
    case "RUNNING":
      return "running";
    case "COMPLETED":
      return "completed";
    case "FAILED":
      return "failed";
    case "PENDING":
    default:
      return "pending";
  }
}

export function createNewJobObject(job: JobFragment): Job {
  return {
    id: job.id,
    deploymentId: job.deploymentId,
    workspaceId: job.workspaceId,
    status: toJobStatus(job.status),
    startedAt: job.startedAt,
    completedAt: job.completedAt,
  };
}
