import { GraphQLClient } from "graphql-request";
import {
  createContext,
  useState,
  ReactNode,
  useEffect,
  useContext,
} from "react";

import { config } from "@flow/config";

import { Sdk, getSdk } from "../__gen__/plugins/graphql-request";

import { requestMiddleware } from "./GraphQLRequestMiddleware";

const GraphQLContext = createContext<Sdk | undefined>(undefined);

export const useGraphQLContext = () => useContext(GraphQLContext);

export const GraphQLRequestProvider = ({
  accesstoken,
  children,
}: {
  accesstoken?: string;
  children?: ReactNode;
}) => {
  const [graphQLSdk, setGraphQLSdk] = useState<Sdk | undefined>();

  const isMockMode =
    import.meta.env.FLOW_ENABLE_MOCK === "true" || config().devMode;
  const endpoint = isMockMode
    ? `${window.location.origin}/api/graphql`
    : `${config().api}/api/graphql`;

  useEffect(() => {
    if (graphQLSdk) return;

    const headers: HeadersInit = {};

    if (accesstoken && !isMockMode) {
      headers.authorization = `Bearer ${accesstoken}`;
    } else if (isMockMode) {
      headers.authorization = "Bearer mock-token";
    }
    const graphQLClient = new GraphQLClient(endpoint, {
      headers,
      requestMiddleware: requestMiddleware,
    });

    const sdk = getSdk(graphQLClient);
    setGraphQLSdk(sdk);
  }, [graphQLSdk, endpoint, accesstoken, setGraphQLSdk, isMockMode]);

  return graphQLSdk ? (
    <GraphQLContext.Provider value={graphQLSdk}>
      {children}
    </GraphQLContext.Provider>
  ) : null;
};
