import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useRef, useEffect, useCallback } from "react";

import { Log } from "@flow/types";

import { LogFragment } from "../__gen__/graphql";
import { RealTimeLogsSubscription } from "../__gen__/plugins/graphql-request";
import { toLog } from "../convert";
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
  const wsClient = useWsClient();
  const queryClient = useQueryClient();
  const isSubscribedRef = useRef(false);

  const query = useQuery<Log[]>({
    queryKey: [LogSubscriptionKeys.GetLogs, jobId],
    queryFn: async () => {
      // This will just retrieve the initial empty array or cached data
      const cachedData = queryClient.getQueryData<Log[]>([
        LogSubscriptionKeys.GetLogs,
        jobId,
      ]);
      return cachedData || [];
    },
    // Important: initial query should run only once
    staleTime: Infinity,
    gcTime: Infinity,
    refetchOnWindowFocus: false,
    refetchOnMount: false,
    refetchOnReconnect: false,
  });

  // Set up subscription separately from React Query
  useEffect(() => {
    // Important: Use ref instead of state to track subscription
    // to prevent re-renders and infinite loops
    if (!jobId || isSubscribedRef.current) return;

    isSubscribedRef.current = true;
    const processedLogIds = new Set<string>();
    let localLogs: Log[] = [];
    let connectionActive = true;
    let updateTimeout: NodeJS.Timeout | null = null;

    // Initialize local logs with any cached data
    const cachedData = queryClient.getQueryData<Log[]>([
      LogSubscriptionKeys.GetLogs,
      jobId,
    ]);
    if (cachedData && cachedData.length > 0) {
      localLogs = [...cachedData];
      cachedData.forEach((log) => {
        // Use 'status' property since this is converted data
        const logId = `${log.message}-${log.status}`;
        processedLogIds.add(logId);
      });
    }

    // Setup connection monitoring
    const heartbeatInterval = setInterval(() => {
      console.log(
        "WebSocket connection check:",
        connectionActive ? "active" : "inactive",
      );
    }, 10000);

    // Function to update query cache
    const updateQueryCache = () => {
      if (updateTimeout) clearTimeout(updateTimeout);

      updateTimeout = setTimeout(() => {
        // Update React Query cache
        queryClient.setQueryData<Log[]>(
          [LogSubscriptionKeys.GetLogs, jobId],
          [...localLogs],
        );
        updateTimeout = null;
      }, 50);
    };

    // Subscribe to logs
    const unsubscribe = wsClient.subscribe<RealTimeLogsSubscription>(
      {
        query: LOG_SUBSCRIPTION,
        variables: { jobId },
      },
      {
        next: (data) => {
          connectionActive = true;

          if (data.data?.logs) {
            // Get log data and transform it
            const rawLog = data.data.logs as LogFragment;
            const logEntry = toLog(rawLog);

            // Create unique ID - IMPORTANT: Use 'status' not 'logLevel' after conversion
            const logId = `${logEntry.message}-${logEntry.status}`;

            // Skip if already processed
            if (processedLogIds.has(logId)) return;

            // Mark as processed
            processedLogIds.add(logId);

            // Add to local logs
            localLogs.push(logEntry);

            // Sort logs by timestamp
            localLogs.sort((a, b) => {
              const dateA = new Date(a.timestamp).getTime();
              const dateB = new Date(b.timestamp).getTime();
              return dateA - dateB;
            });

            // Update React Query cache
            updateQueryCache();
          }
        },
        error: (err) => {
          console.error("Subscription error:", err);
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
      clearInterval(heartbeatInterval);
      if (updateTimeout) clearTimeout(updateTimeout);
      unsubscribe();
      isSubscribedRef.current = false;
    };
  }, [jobId, wsClient, queryClient]); // Removed isSubscribed from dependencies

  // Function to clear logs
  const clearLogs = useCallback(() => {
    queryClient.setQueryData<Log[]>([LogSubscriptionKeys.GetLogs, jobId], []);
  }, [jobId, queryClient]);

  return {
    ...query,
    clearLogs,
  };
};
