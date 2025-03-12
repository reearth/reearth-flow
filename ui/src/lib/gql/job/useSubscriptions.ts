import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useRef, useEffect, useCallback } from "react";

import { JobStatus } from "@flow/types";

import { OnJobStatusChangeSubscription } from "../__gen__/graphql";
import { toJobStatus } from "../convert";
import { useWsClient } from "../provider/GraphQLSubscriptionProvider";

const JOB_STATUS_SUBSCRIPTION = `
 subscription OnJobStatusChange($jobId: ID!) {
   jobStatus(jobId: $jobId)
 }
`;

export enum JobSubscriptionKeys {
  GetJobStatus = "getJobStatus",
}

export const useJobStatus = (jobId: string) => {
  const wsClient = useWsClient();
  const queryClient = useQueryClient();
  const isSubscribedRef = useRef(false);
  const unSubscribedRef = useRef<(() => void) | undefined>(undefined);

  const query = useQuery({
    queryKey: [JobSubscriptionKeys.GetJobStatus, jobId],
    queryFn: async () => {
      const cachedData = queryClient.getQueryData<JobStatus>([
        JobSubscriptionKeys.GetJobStatus,
        jobId,
      ]);
      return cachedData || {};
    },
    // Important: initial query should run only once
    staleTime: Infinity,
    gcTime: Infinity,
    refetchOnWindowFocus: false,
    refetchOnMount: false,
    refetchOnReconnect: false,
  });

  useEffect(() => {
    if (!jobId || isSubscribedRef.current) return;

    isSubscribedRef.current = true;

    // Subscribe to job status
    const unsubscribe = wsClient.subscribe<OnJobStatusChangeSubscription>(
      {
        query: JOB_STATUS_SUBSCRIPTION,
        variables: { jobId },
      },
      {
        next: (data) => {
          if (data.data?.jobStatus) {
            const newStatus = data.data.jobStatus;
            // Update React Query cache
            queryClient.setQueryData(
              [JobSubscriptionKeys.GetJobStatus, jobId],
              toJobStatus(newStatus),
            );
          }
        },
        error: (err) => {
          console.error(`Status subscription error ${jobId}:`, err);
        },
        complete: () => {
          console.info("Status Subscription complete");
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
  }, [jobId, wsClient, queryClient]);

  const stopSubscription = useCallback(async () => {
    if (unSubscribedRef.current) {
      unSubscribedRef.current();
      isSubscribedRef.current = false;
    }
  }, []);

  return {
    ...query,
    isSubscribedRef,
    stopSubscription,
  };
};
