import { GraphQLClient } from "graphql-request";
import { createContext, useState, ReactNode, useEffect } from "react";

import { Loading } from "@flow/components";
import { useAuth } from "@flow/lib/auth";
import { Sdk, getSdk } from "@flow/lib/gql/";

// TODO: is there any other way without exporting the context?
const defaultSdk = getSdk(new GraphQLClient(""));
export const GraphQlSdkContext = createContext(defaultSdk);

export const GraphQlSdkProvider = ({ children }: { children?: ReactNode }) => {
  const [graphQLSdk, setGraphQLSdk] = useState<Sdk | undefined>();
  const endpoint = `${window.FLOW_CONFIG?.api}/graphql`;
  const { getAccessToken } = useAuth();

  // TODO: What happens when the token expires?
  // Maybe parse the token, if it's expired get the token again?
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

  return graphQLSdk && children ? (
    <GraphQlSdkContext.Provider value={graphQLSdk}>{children}</GraphQlSdkContext.Provider>
  ) : (
    <Loading />
  );
};
