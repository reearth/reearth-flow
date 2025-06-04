import { GraphQLClient } from "graphql-request";
import {
  createContext,
  useState,
  ReactNode,
  useEffect,
  useContext,
} from "react";

import { Sdk, getSdk } from "../__gen__/plugins/graphql-request";

import { requestMiddleware } from "./GraphQLRequestMiddleware";

const DocumentGraphQLContext = createContext<Sdk | undefined>(undefined);

export const useDocumentGraphQLContext = () => useContext(DocumentGraphQLContext);

export const DocumentGraphQLProvider = ({
  accesstoken,
  children,
}: {
  accesstoken?: string;
  children?: ReactNode;
}) => {
  const [graphQLSdk, setGraphQLSdk] = useState<Sdk | undefined>();
  // Fixed endpoint for document operations
  const endpoint = "http://127.0.0.1:8080/api/graphql";

  useEffect(() => {
    if (graphQLSdk) return;

    const headers: HeadersInit = {};

    if (accesstoken) {
      headers.authorization = `Bearer ${accesstoken}`;
    }
    const graphQLClient = new GraphQLClient(endpoint, {
      headers,
      requestMiddleware: requestMiddleware,
    });

    const sdk = getSdk(graphQLClient);
    setGraphQLSdk(sdk);
  }, [graphQLSdk, endpoint, accesstoken, setGraphQLSdk]);

  return graphQLSdk ? (
    <DocumentGraphQLContext.Provider value={graphQLSdk}>
      {children}
    </DocumentGraphQLContext.Provider>
  ) : null;
}; 