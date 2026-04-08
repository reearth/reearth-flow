import { useCallback, useEffect, useRef } from "react";

import {
  OnJobStatusChangeSubscription,
  UserFacingLogFragment,
  UserFacingLogsSubscription,
} from "@flow/lib/gql/__gen__/graphql";
import { toJobStatus, toUserFacingLog } from "@flow/lib/gql/convert";
import { useSubscription } from "@flow/lib/gql/subscriptions/useSubscription";
import { useSubscriptionSetup } from "@flow/lib/gql/subscriptions/useSubscriptionSetup";
import { useIndexedDB } from "@flow/lib/indexedDB";
import { UserFacingLog } from "@flow/types";

export default (accessToken?: string, jobId?: string) => {
  const processedLogIds = useRef(new Set<string>());

  const { value: debugRunState, updateValue } = useIndexedDB("debugRun");

  const debugRun = debugRunState?.jobs?.find((job) => job.jobId === jobId);

  useEffect(() => {
    if (!jobId && processedLogIds.current.size > 0) {
      processedLogIds.current.clear();
    }
  }, [jobId]);

  const variables = { jobId };

  const userFacingLogsDataFormatter = useCallback(
    (
      data: UserFacingLogsSubscription,
      cachedData?: UserFacingLog[] | undefined,
    ) => {
      if (data?.userFacingLogs && (!cachedData || Array.isArray(cachedData))) {
        const cachedLogs = [...(cachedData ?? [])];
        const rawLog = data.userFacingLogs as UserFacingLogFragment;
        const logEntry = toUserFacingLog(rawLog);

        const logId = `${logEntry.message}-${logEntry.level}-${logEntry.timestamp}`;
        if (processedLogIds.current.has(logId)) return;
        processedLogIds.current.add(logId);

        cachedLogs.push(logEntry);
        cachedLogs.sort((a, b) => {
          const dateA = new Date(a.timestamp).getTime();
          const dateB = new Date(b.timestamp).getTime();
          return dateA - dateB;
        });

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

  // Keep IndexedDB status in sync so port intermediate data checks work
  useEffect(() => {
    if (!jobId || !debugRun) return;
    if (debugRun.status !== realTimeJobStatus) {
      updateValue((prevState) => {
        const jobs = prevState.jobs.map((job) => {
          if (job.jobId !== jobId) return job;
          return {
            ...job,
            status: realTimeJobStatus as any,
          };
        });
        return { jobs };
      });
    }
  }, [realTimeJobStatus, debugRun, jobId, updateValue]);
};
