import { useQuery } from "@tanstack/react-query";

import { useGraphQLContext } from "@flow/lib/gql";

import { graphql } from "../__gen__";

// Need the queries to build the plugin
graphql(`
  query GetMe {
    me {
      id
      name
      email
      myWorkspaceId
    }
  }
`);

export enum UserQueryKeys {
  GetMe = "getMe",
}

export const useMeQuery = () => {
  const graphQLContext = useGraphQLContext();
  return useQuery({
    queryKey: [UserQueryKeys.GetMe],
    queryFn: async () => graphQLContext?.GetMe(),
  });
};
