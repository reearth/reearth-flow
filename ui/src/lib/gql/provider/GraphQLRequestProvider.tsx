import { GraphQLClient } from "graphql-request";
import { createContext, useState, ReactNode, useEffect, useContext } from "react";

import { config } from "@flow/config";
import { useAuth } from "@flow/lib/auth";

import { Sdk, getSdk } from "../__gen__/plugins/graphql-request";

export const GraphQLContext = createContext<Sdk | undefined>(undefined);

export const useGraphQLContext = () => useContext(GraphQLContext);

export const GraphQLRequestProvider = ({ children }: { children?: ReactNode }) => {
  const [graphQLSdk, setGraphQLSdk] = useState<Sdk | undefined>();
  const endpoint = `${config().api}/graphql`;
  const { getAccessToken } = useAuth();

  useEffect(() => {
    if (graphQLSdk) return;
    (async () => {
      const token = await getAccessToken();

      const graphQLClient = new GraphQLClient(endpoint, {
        headers: {
          authorization: `Bearer ${token}`,
        },
      });
      const sdk = getSdk(graphQLClient);
      setGraphQLSdk(sdk);
    })();
  }, [graphQLSdk, setGraphQLSdk, getAccessToken, endpoint]);

  return graphQLSdk ? (
    <GraphQLContext.Provider value={graphQLSdk}>{children}</GraphQLContext.Provider>
  ) : (
    children
  );
};
