import { useEffect, useMemo, useRef } from "react";

import {
  LogFragment,
  OnJobStatusChangeSubscription,
  RealTimeLogsSubscription,
} from "@flow/lib/gql/__gen__/graphql";
import { toJobStatus, toLog } from "@flow/lib/gql/convert";
import { useSubscriptionSetup } from "@flow/lib/gql/subscriptions/useSubscriptionSetup";
import { useIndexedDB } from "@flow/lib/indexedDB";
import { useCurrentProject } from "@flow/stores";
import { Log } from "@flow/types";

export default (accessToken?: string) => {
  const [currentProject] = useCurrentProject();
  const { value: debugRunState } = useIndexedDB("debugRun");

  const currentDebugJobId = useMemo(
    () =>
      debugRunState?.jobs?.find((job) => job.projectId === currentProject?.id)
        ?.jobId,
    [debugRunState, currentProject],
  );

  const processedLogIds = useRef(new Set<string>());

  useSubscriptionSetup<OnJobStatusChangeSubscription>(
    "GetSubscribedJobStatus",
    accessToken,
    { jobId: currentDebugJobId },
    currentDebugJobId,
    (data) => toJobStatus(data.jobStatus),
    !currentDebugJobId,
  );
  useSubscriptionSetup<RealTimeLogsSubscription, Log[]>(
    "GetSubscribedLogs",
    accessToken,
    { jobId: currentDebugJobId },
    currentDebugJobId,
    (data, cachedData) => {
      if (data?.logs && (!cachedData || Array.isArray(cachedData))) {
        const cachedLogs = [...(cachedData ?? [])];
        // Get log data and transform it
        const rawLog = data.logs as LogFragment;
        const logEntry = toLog(rawLog);

        // Create unique ID - IMPORTANT: Use 'status' not 'logLevel' after conversion
        const logId = `${logEntry.message}-${logEntry.status}`;

        // Skip if already processed
        if (processedLogIds.current.has(logId)) return;

        // Mark as processed
        processedLogIds.current.add(logId);

        // Add to local logs
        cachedLogs.push(logEntry);

        // Sort logs by timestamp
        cachedLogs.sort((a, b) => {
          const dateA = new Date(a.timestamp).getTime();
          const dateB = new Date(b.timestamp).getTime();
          return dateA - dateB;
        });

        // Update React Query cache
        return [...cachedLogs];
      }
    },
    !currentDebugJobId,
  );

  useEffect(() => {
    if (!currentDebugJobId && processedLogIds.current.size > 0) {
      processedLogIds.current.clear();
    }
  }, [currentDebugJobId]);
};
