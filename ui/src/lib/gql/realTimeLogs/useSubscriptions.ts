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
        let initialDataReceived = false;
        let connectionActive = true;

        // Add connection monitoring
        const heartbeatInterval = setInterval(() => {
          console.log(
            "WebSocket connection check:",
            connectionActive ? "active" : "inactive",
          );
        }, 10000);

        const unsubscribe = wsClient.subscribe<RealTimeLogsSubscription>(
          {
            query: LOG_SUBSCRIPTION,
            variables: { jobId },
          },
          {
            next: (data) => {
              console.log(
                "Received log data packet:",
                new Date().toISOString(),
              );
              connectionActive = true;

              if (data.data) {
                // Update the cache with new data
                queryClient.setQueryData(
                  [LogSubscriptionKeys.GetLogs, jobId],
                  (oldData: any) => {
                    console.log("old data", oldData);
                    const newLogs = data.data?.logs;
                    if (!oldData) return newLogs;

                    return {
                      ...oldData,
                      logs: [...(oldData.logs || []), newLogs],
                    };
                  },
                );

                if (!initialDataReceived) {
                  initialDataReceived = true;
                  resolve(data.data.logs);
                }
              }
            },
            error: (error) => {
              console.error("Subscription error", error);
              if (!initialDataReceived) {
                reject(error);
              }
            },
            complete: () => {
              console.log("Subscription complete");
              connectionActive = false;
              clearInterval(heartbeatInterval);
              // Handle completion
            },
          },
        );

        return () => {
          clearInterval(heartbeatInterval);
          unsubscribe();
        };
      }),
    refetchInterval: false,
    retry: 3,
    staleTime: Infinity,
    gcTime: Infinity, // Keep the cache since we're updating it via subscription
  });
};
