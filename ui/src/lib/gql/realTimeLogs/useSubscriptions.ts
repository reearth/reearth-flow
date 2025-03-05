import { useQuery, useQueryClient } from "@tanstack/react-query";

import { RealTimeLogsSubscription } from "../__gen__/graphql";
import { useWsClient } from "../subscriptions";

const LOG_SUBSCRIPTION = `
 subscription RealTimeLogs($jobId: ID!) {
   logs(jobId: $jobId) {
   jobId
   nodeId
   logLevel
   message
   }
 }
`;

export enum LogSubscriptionKeys {
  GetLogs = "getLogs",
}

export const useLogs = (jobId: string) => {
  const queryClient = useQueryClient();
  const wsClient = useWsClient();

  return useQuery({
    queryKey: [LogSubscriptionKeys.GetLogs, jobId],
    queryFn: () =>
      new Promise<RealTimeLogsSubscription["logs"]>((resolve, reject) => {
        let active = true;

        const unsubscribe = wsClient.subscribe<RealTimeLogsSubscription>(
          {
            query: LOG_SUBSCRIPTION,
            variables: { jobId },
          },
          {
            next: (data) => {
              console.log("next", data);
              if (data.data) {
                // Update the cache with new data
                queryClient.setQueryData(["logs", jobId], data.data);

                // Only resolve the initial promise if we haven't already
                if (active) {
                  active = false;
                  resolve(data.data.logs);
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

        console.log("subscribe", active);

        return () => {
          active = false;
          unsubscribe();
        };
      }),
    refetchInterval: false,
    retry: 3,
    staleTime: Infinity,
    gcTime: Infinity, // Keep the cache since we're updating it via subscription
  });
};
