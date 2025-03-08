import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useRef, useEffect, useCallback } from "react";

import { Log } from "@flow/types";

import { LogFragment } from "../__gen__/graphql";
import { RealTimeLogsSubscription } from "../__gen__/plugins/graphql-request";
import { toLog } from "../convert";
import { useWsClient } from "../provider/GraphQLSubscriptionProvider";

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
  const unSubscribedRef = useRef<(() => void) | undefined>(undefined);

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

    // Subscribe to logs
    const unsubscribe = wsClient.subscribe<RealTimeLogsSubscription>(
      {
        query: LOG_SUBSCRIPTION,
        variables: { jobId },
      },
      {
        next: (data) => {
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
            queryClient.setQueryData<Log[]>(
              [LogSubscriptionKeys.GetLogs, jobId],
              [...localLogs],
            );
          }
        },
        error: (err) => {
          console.error("Subscription error:", err);
        },
        complete: () => {
          console.info("Subscription complete");
          isSubscribedRef.current = false;
        },
      },
    );

    unSubscribedRef.current = unsubscribe;

    // Cleanup
    return () => {
      unsubscribe();
      isSubscribedRef.current = false;
    };
  }, [jobId, wsClient, queryClient]); // Removed isSubscribed from dependencies

  // Function to clear logs
  const clearLogs = useCallback(() => {
    queryClient.setQueryData<Log[]>([LogSubscriptionKeys.GetLogs, jobId], []);
  }, [jobId, queryClient]);

  const stopSubscription = useCallback(async () => {
    unSubscribedRef.current?.();
  }, []);

  return {
    ...query,
    isSubscribedRef,
    clearLogs,
    stopSubscription,
  };
};
