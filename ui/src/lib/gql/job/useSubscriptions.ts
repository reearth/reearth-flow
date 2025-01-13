import { useQuery, useQueryClient } from "@tanstack/react-query";

import { JobStatusSubscription } from "../__gen__/graphql";
import { wsClient } from "../subscriptions";

import { JobQueryKeys } from "./useQueries";

const JOB_STATUS_SUBSCRIPTION = `
 subscription OnJobStatusChange($jobId: ID!) {
   jobStatus(jobId: $jobId)
 }
`;

export const useJobStatus = (jobId: string) => {
  const queryClient = useQueryClient();

  return useQuery({
    queryKey: [JobQueryKeys.GetJobStatus, jobId],
    queryFn: () =>
      new Promise<JobStatusSubscription["jobStatus"]>((resolve, reject) => {
        let active = true;

        const unsubscribe = wsClient.subscribe<
          JobStatusSubscription["jobStatus"]
        >(
          {
            query: JOB_STATUS_SUBSCRIPTION,
            variables: { jobId },
          },
          {
            next: (data) => {
              console.log("next", data);
              if (data.data) {
                // Update the cache with new data
                queryClient.setQueryData(["jobStatus", jobId], data.data);

                // Only resolve the initial promise if we haven't already
                if (active) {
                  active = false;
                  resolve(data.data);
                }
              }
            },
            error: (error) => {
              if (active) {
                reject(error);
              }
            },
            complete: () => {
              console.log("complete");
              // Handle completion
            },
          },
        );

        return () => {
          active = false;
          console.log("unsubscribe");
          unsubscribe();
        };
      }),
    refetchInterval: false,
    retry: 3,
    staleTime: Infinity,
    gcTime: Infinity, // Keep the cache since we're updating it via subscription
  });
};
