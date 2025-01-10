import { createClient } from "graphql-ws";

import { config } from "@flow/config";

export const wsClient = createClient({
  url: `${config().api}/api/graphql`,
  retryAttempts: 5,
  shouldRetry: () => true,
  connectionParams: () => ({
    authorization: `Bearer ${localStorage.getItem("token")}`,
  }),
});
