import { GraphQLClient } from "graphql-request";
import { createContext, useState, ReactNode, useEffect } from "react";

import { Loading } from "@flow/components";
import { useAuth } from "@flow/lib/auth";

export const GraphQlContext = createContext<GraphQLClient>(new GraphQLClient(""));

export const GraphQlClientProvider = ({ children }: { children?: ReactNode }) => {
  const [graphQLClient, setGraphQLClient] = useState<any | undefined>();
  const endpoint = `${window.FLOW_CONFIG?.api}/graphql`;
  const { getAccessToken } = useAuth();

  // TODO: What happens when the token expires?
  // Maybe parse the token, if it's expired get the token again?
  useEffect(() => {
    if (graphQLClient) return;
    (async () => {
      const token = await getAccessToken();

      const graphQLClient = new GraphQLClient(endpoint, {
        headers: {
          authorization: `Bearer ${token}`,
        },
      });
      setGraphQLClient(graphQLClient);
    })();
  }, [graphQLClient, setGraphQLClient, getAccessToken, endpoint]);

  return graphQLClient && children ? (
    <GraphQlContext.Provider value={graphQLClient}>{children}</GraphQlContext.Provider>
  ) : (
    <Loading />
  );
};
