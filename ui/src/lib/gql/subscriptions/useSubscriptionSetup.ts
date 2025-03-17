import { useQueryClient } from "@tanstack/react-query";
import { Client, createClient } from "graphql-ws";
import { useEffect, useRef } from "react";

import { config } from "@flow/config";

export enum SubscriptionKeys {
  GetSubscribedLogs = "getSubscribedLogs",
  GetSubscribedJobStatus = "getSubscribedJobStatus",
  GetSubscribedEdgeStatus = "getSubscribedEdgeStatus",
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

const EDGE_STATUS_SUBSCRIPTION = `
  subscription OnEdgeStatusChange($jobId: ID!, $edgeId: String!) {
    edgeStatus(jobId: $jobId, edgeId: $edgeId)
  }
`;

const SubscriptionStrings: Record<PossibleSubscriptionKeys, string> = {
  GetSubscribedJobStatus: JOB_STATUS_SUBSCRIPTION,
  GetSubscribedEdgeStatus: EDGE_STATUS_SUBSCRIPTION,
  GetSubscribedLogs: LOG_SUBSCRIPTION,
};

const getWebSocketClient = (disabled?: boolean) => {
  const clients = new Map<string, Client>();

  return (
    url: string,
    key: string,
    accessToken?: string,
  ): Client | undefined => {
    if (!accessToken || disabled) return undefined;

    if (!clients.has(key)) {
      const newClient = createClient({
        url,
        retryAttempts: 5,
        shouldRetry: () => true,
        connectionParams: () => ({
          headers: {
            authorization: `Bearer ${accessToken}`,
          },
        }),
        lazy: true,
      });

      clients.set(key, newClient);
      return newClient;
    }

    const client = clients.get(key);

    return client;
  };
};

export function useSubscriptionSetup<Data = any, CachedData = any>(
  subscriptionKey: PossibleSubscriptionKeys,
  accessToken: string | undefined,
  variables: Record<string, unknown>,
  secondaryCacheKey?: string,
  dataFormatter?: (data: Data, cachedData?: CachedData) => unknown | undefined,
  disabled?: boolean,
) {
  const api = config().api;

  const isSubscribedRef = useRef(false);
  const queryClient = useQueryClient();
  const wsClient = getWebSocketClient(disabled)(
    `${api}/api/graphql`,
    `${subscriptionKey}:${secondaryCacheKey}`,
    accessToken,
  );

  useEffect(() => {
    if (isSubscribedRef.current || disabled) return;

    isSubscribedRef.current = true;
    // Set up subscription only once
    const unsubscribe = wsClient?.subscribe<Data>(
      {
        query: SubscriptionStrings[subscriptionKey],
        variables,
      },
      {
        next: (data) => {
          if (data.data) {
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
          unsubscribe?.();
        },
      },
    );

    return () => {
      isSubscribedRef.current = false;
      unsubscribe?.();
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
