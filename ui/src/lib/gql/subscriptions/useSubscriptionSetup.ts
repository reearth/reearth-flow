import { useQueryClient } from "@tanstack/react-query";
import { createClient } from "graphql-ws";
import { useEffect, useRef } from "react";

import { config } from "@flow/config";

export enum SubscriptionKeys {
  GetSubscribedLogs = "getSubscribedLogs",
  GetSubscribedJobStatus = "getSubscribedJobStatus",
}

export type PossibleSubscriptionKeys = keyof typeof SubscriptionKeys;

const JOB_STATUS_SUBSCRIPTION = `
 subscription OnJobStatusChange($jobId: ID!) {
   jobStatus(jobId: $jobId)
 }
`;

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

const SubscriptionStrings: Record<PossibleSubscriptionKeys, string> = {
  GetSubscribedJobStatus: JOB_STATUS_SUBSCRIPTION,
  GetSubscribedLogs: LOG_SUBSCRIPTION,
};

export function useSubscriptionSetup<Data = any, CachedData = any>(
  subscriptionKey: PossibleSubscriptionKeys,
  accessToken: string | undefined,
  variables?: Record<string, unknown>,
  secondaryCacheKey?: string,
  dataFormatter?: (data: Data, cachedData?: CachedData) => unknown | undefined,
  disabled?: boolean,
) {
  const isSubscribedRef = useRef(false);
  const queryClient = useQueryClient();

  const api = config().api;

  const wsClient = createClient({
    url: `${api}/api/graphql`,
    retryAttempts: 5,
    shouldRetry: () => true,
    connectionParams: () => {
      return {
        headers: {
          authorization: accessToken ? `Bearer ${accessToken}` : "",
        },
      };
    },
  });

  useEffect(() => {
    if (isSubscribedRef.current || disabled || !variables) return;

    isSubscribedRef.current = true;
    // Set up subscription only once
    const unsubscribe = wsClient.subscribe<Data>(
      {
        query: SubscriptionStrings[subscriptionKey],
        variables,
      },
      {
        next: (data) => {
          if (data.data) {
            console.log(
              `Subscription data for ${subscriptionKey}: `,
              data.data,
            );
            // Update React Query cache
            const cachedData = queryClient.getQueryData<CachedData>([
              SubscriptionKeys[subscriptionKey],
              secondaryCacheKey,
            ]);

            const formattedData = dataFormatter
              ? dataFormatter(data.data, cachedData)
              : data.data;

            queryClient.setQueryData(
              [SubscriptionKeys[subscriptionKey], secondaryCacheKey],
              formattedData,
            );
          }
        },
        error: (err) => {
          console.error(`Subscription error for ${subscriptionKey}: `, err);
        },
        complete: () => {
          console.info(`Subscription completed for ${subscriptionKey}`);
          isSubscribedRef.current = false;
        },
      },
    );

    return () => {
      console.log("unsubscribing", subscriptionKey, variables);
      isSubscribedRef.current = false;
      unsubscribe();
    };
  }, [
    disabled,
    variables,
    secondaryCacheKey,
    subscriptionKey,
    wsClient,
    queryClient,
    dataFormatter,
  ]);
}
