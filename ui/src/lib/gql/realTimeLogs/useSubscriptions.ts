import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useRef, useEffect } from "react";

import { RealTimeLogsSubscription } from "../__gen__/plugins/graphql-request";
import { useWsClient } from "../subscriptions";

const LOG_SUBSCRIPTION = `
 subscription RealTimeLogs($jobId: ID!) {
   logs(jobId: $jobId) {
     jobId
     nodeId
     timestamp
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
  const processedLogsRef = useRef(new Set<string>());
  const logsArrayRef = useRef<RealTimeLogsSubscription["logs"][]>([]);
  const updateTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const isSubscribedRef = useRef(false);

  // Setup subscription outside of useQuery to prevent multiple subscriptions
  useEffect(() => {
    if (!jobId || isSubscribedRef.current) return;

    isSubscribedRef.current = true;
    let connectionActive = true;

    // Setup connection monitoring
    const heartbeatInterval = setInterval(() => {
      console.log(
        "WebSocket connection check:",
        connectionActive ? "active" : "inactive",
      );
    }, 10000);

    // Function to batch updates to the cache
    const updateQueryCache = () => {
      if (updateTimeoutRef.current) {
        clearTimeout(updateTimeoutRef.current);
      }

      updateTimeoutRef.current = setTimeout(() => {
        queryClient.setQueryData(
          [LogSubscriptionKeys.GetLogs, jobId],
          [...logsArrayRef.current],
        );
        updateTimeoutRef.current = null;
      }, 50);
    };

    const unsubscribe = wsClient.subscribe<RealTimeLogsSubscription>(
      {
        query: LOG_SUBSCRIPTION,
        variables: { jobId },
      },
      {
        next: (data) => {
          connectionActive = true;

          if (data.data?.logs) {
            const logEntry = data.data.logs;

            // Create a unique identifier for this log
            const logId = `${logEntry.message}-${logEntry.logLevel}`;

            // Skip if we've seen this log before
            if (processedLogsRef.current.has(logId)) {
              return;
            }

            // Mark as processed
            processedLogsRef.current.add(logId);

            // Add to our array
            logsArrayRef.current.push(logEntry);

            // Update cache (debounced)
            updateQueryCache();
          }
        },
        error: (error) => {
          console.error("Subscription error", error);
          connectionActive = false;
        },
        complete: () => {
          console.log("Subscription complete");
          connectionActive = false;
          isSubscribedRef.current = false;
          clearInterval(heartbeatInterval);
        },
      },
    );

    // Cleanup
    return () => {
      isSubscribedRef.current = false;
      clearInterval(heartbeatInterval);
      if (updateTimeoutRef.current) {
        clearTimeout(updateTimeoutRef.current);
        updateTimeoutRef.current = null;
      }
      unsubscribe();
    };
  }, [jobId, wsClient, queryClient]);

  // Use React Query to access the logs
  const query = useQuery({
    queryKey: [LogSubscriptionKeys.GetLogs, jobId],
    queryFn: () => {
      // This will resolve immediately if we already have logs
      if (logsArrayRef.current.length > 0) {
        return Promise.resolve(logsArrayRef.current);
      }

      // Otherwise create a promise that will resolve when logs arrive
      return new Promise<RealTimeLogsSubscription["logs"][]>((resolve) => {
        // Check if we have logs every 100ms
        const checkInterval = setInterval(() => {
          if (logsArrayRef.current.length > 0) {
            clearInterval(checkInterval);
            resolve(logsArrayRef.current);
          }
        }, 100);

        // Timeout after 5 seconds to prevent hanging
        setTimeout(() => {
          clearInterval(checkInterval);
          resolve(logsArrayRef.current);
        }, 5000);
      });
    },
    refetchInterval: false,
    staleTime: Infinity,
    gcTime: Infinity,
  });

  // Function to clear logs
  const clearLogs = () => {
    logsArrayRef.current = [];
    processedLogsRef.current.clear();
    queryClient.setQueryData([LogSubscriptionKeys.GetLogs, jobId], []);
  };

  return {
    ...query,
    clearLogs,
  };
};
