import type { PaginationOptions } from "@flow/types/paginationOptions";

import { useQueries } from "./useQueries";

export const useJob = () => {
  const { useGetJobsQuery, useGetJobQuery } = useQueries();

  const useGetJobs = (
    workspaceId?: string,
    paginationOptions?: PaginationOptions,
  ) => {
    const { data, ...rest } = useGetJobsQuery(workspaceId, paginationOptions);
    return {
      pages: data,
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
    useGetJobs,
  };
};
