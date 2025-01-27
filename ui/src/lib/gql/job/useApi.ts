import { useQueries } from "./useQueries";

export const useJob = () => {
  const { useGetJobsInfiniteQuery, useGetJobQuery } = useQueries();

  const useGetJobsInfinite = (workspaceId?: string, fetchRate?: number) => {
    const { data, ...rest } = useGetJobsInfiniteQuery(workspaceId, fetchRate);
    return {
      pages: data?.pages,
      ...rest,
    };
  };

  const useGetJob = (jobId: string) => {
    const { data, ...rest } = useGetJobQuery(jobId);
    return {
      job: data,
      ...rest,
    };
  };

  return {
    useGetJob,
    useGetJobsInfinite,
  };
};
