import { createClient } from "graphql-ws";
import { useMemo } from "react";

import { config } from "@flow/config";
import { useAuth } from "@flow/lib/auth";

export const useWsClient = () => {
  const api = config().api;
  const { getAccessToken } = useAuth();
  return useMemo(() => {
    return createClient({
      url: `${api}/api/graphql`,
      retryAttempts: 5,
      shouldRetry: () => true,
      connectionParams: async () => {
        const token = await getAccessToken();
        return {
          headers: {
            authorisation: `Bearer ${token}`,
          },
        };
      },
    });
  }, [api, getAccessToken]);
};
