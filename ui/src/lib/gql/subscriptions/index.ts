import { createClient } from "graphql-ws";
import { useMemo } from "react";

import { config } from "@flow/config";

export const useWsClient = () => {
  const api = config().api;

  return useMemo(() => {
    return createClient({
      url: `${api}/api/graphql`,

      retryAttempts: 5,
      shouldRetry: () => true,
      connectionParams: () => ({
        authorization: `Bearer ${localStorage.getItem("token")}`,
      }),
    });
  }, [api]);
};
