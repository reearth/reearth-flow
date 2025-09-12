import { useCallback, useEffect, useMemo, useRef } from "react";

import {
  OnJobStatusChangeSubscription,
  UserFacingLogFragment,
  UserFacingLogsSubscription,
} from "@flow/lib/gql/__gen__/graphql";
import { toJobStatus, toUserFacingLog } from "@flow/lib/gql/convert";
import { useSubscription } from "@flow/lib/gql/subscriptions/useSubscription";
import { useSubscriptionSetup } from "@flow/lib/gql/subscriptions/useSubscriptionSetup";
import { useIndexedDB } from "@flow/lib/indexedDB";
import { JobStatus, UserFacingLog } from "@flow/types";

export default (accessToken?: string, jobId?: string, projectId?: string) => {
  const processedLogIds = useRef(new Set<string>());

  const { value: debugRunState, updateValue } = useIndexedDB("debugRun");

  const debugRun = useMemo(
    () => debugRunState?.jobs?.find((job) => job.projectId === projectId),
    [debugRunState, projectId],
  );

  useEffect(() => {
    if (!jobId && processedLogIds.current.size > 0) {
      processedLogIds.current.clear();
    }
  }, [jobId]);

  const variables = useMemo(() => ({ jobId }), [jobId]);

  const userFacingLogsDataFormatter = useCallback(
    (
      data: UserFacingLogsSubscription,
      cachedData?: UserFacingLog[] | undefined,
    ) => {
      if (data?.userFacingLogs && (!cachedData || Array.isArray(cachedData))) {
        const cachedLogs = [...(cachedData ?? [])];
        // Get log data and transform it
        const rawLog = data.userFacingLogs as UserFacingLogFragment;
        const logEntry = toUserFacingLog(rawLog);

        // Create unique ID - IMPORTANT: Use 'status' not 'logLevel' after conversion
        const logId = `${logEntry.message}-${logEntry.level}-${logEntry.timestamp}`;

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
    [],
  );

  const jobStatusDataFormatter = useCallback(
    (data: OnJobStatusChangeSubscription) => {
      return toJobStatus(data.jobStatus);
    },
    [],
  );

  useSubscriptionSetup<OnJobStatusChangeSubscription>(
    "GetSubscribedJobStatus",
    accessToken,
    variables,
    jobId,
    jobStatusDataFormatter,
    !jobId || debugRun?.status === "completed" || debugRun?.status === "failed",
  );

  const { data: realTimeJobStatus } = useSubscription(
    "GetSubscribedJobStatus",
    jobId,
    !jobId || debugRun?.status === "completed" || debugRun?.status === "failed",
  );

  useSubscriptionSetup<UserFacingLogsSubscription, UserFacingLog[]>(
    "GetSubscribedUserFacingLogs",
    accessToken,
    variables,
    jobId,
    userFacingLogsDataFormatter,
    !jobId,
  );

  useEffect(() => {
    if (!projectId) return;

    if (debugRun?.status !== realTimeJobStatus) {
      updateValue((prevState) => {
        const jobs = prevState.jobs.map((job) => {
          if (job.projectId === projectId) {
            return {
              ...job,
              status: realTimeJobStatus as any as JobStatus, // This type assertion can be removed if useIndexedDB's updateValue's types get improved
            };
          }
          return job;
        });

        return { jobs };
      });
    }
  }, [realTimeJobStatus, debugRun, projectId, updateValue]);
};
