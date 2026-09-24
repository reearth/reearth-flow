import { GraphQLClient } from "graphql-request";
import type { ReactNode } from "react";
import { createContext, useState, useEffect, useContext } from "react";

import { config } from "@flow/config";

import type { Sdk } from "../__gen__/plugins/graphql-request";
import { getSdk } from "../__gen__/plugins/graphql-request";

import { requestMiddleware } from "./GraphQLRequestMiddleware";

const GraphQLContext = createContext<Sdk | undefined>(undefined);

/**
 * The same client the SDK is built on, for the rare query whose shape is not
 * known at build time and so cannot be generated — see `nodeDiagnosticsBatch`,
 * which aliases one field per node id.
 */
const GraphQLClientContext = createContext<GraphQLClient | undefined>(
  undefined,
);

export const useGraphQLContext = () => useContext(GraphQLContext);

export const useGraphQLClient = () => useContext(GraphQLClientContext);

export const GraphQLRequestProvider = ({
  accesstoken,
  children,
}: {
  accesstoken?: string;
  children?: ReactNode;
}) => {
  const [graphQLSdk, setGraphQLSdk] = useState<Sdk | undefined>();
  const [graphQLClient, setGraphQLClient] = useState<
    GraphQLClient | undefined
  >();

  const isMockMode = config().mockEnabled;
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
      requestMiddleware: requestMiddleware(headers),
    });

    const sdk = getSdk(graphQLClient);
    setGraphQLClient(graphQLClient);
    setGraphQLSdk(sdk);
  }, [graphQLSdk, endpoint, accesstoken, setGraphQLSdk, isMockMode]);

  return graphQLSdk ? (
    <GraphQLContext.Provider value={graphQLSdk}>
      <GraphQLClientContext.Provider value={graphQLClient}>
        {children}
      </GraphQLClientContext.Provider>
    </GraphQLContext.Provider>
  ) : null;
};
