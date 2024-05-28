import { GraphQLClient } from "graphql-request";
import { useState } from "react";

import { e2eAccessToken } from "@flow/config";
import { useAuth } from "@flow/lib/auth";

const endpoint = `${window.FLOW_CONFIG?.api}/graphql`;

// TODO: This is working but incorrectly
// This needs to be initialized once and used everywhere and initialize with the token being there for sure.
// Also, the first request always fails because the token is undefined at that time
export const useClient = () => {
  const { getAccessToken } = useAuth();
  const [token, setToken] = useState<string | undefined>();

  (async () => {
    const token = await getAccessToken();
    setToken(token);
  })();

  const graphQLClient = new GraphQLClient(endpoint, {
    headers: {
      authorization: `Bearer ${e2eAccessToken() || token}`,
    },
  });
  return graphQLClient;
};
