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
  const { data, ...rest } = useQuery({
    // TODO: Use static keys rather than strings. Export the keys as well
    queryKey: [UserQueryKeys.GetMe],
    queryFn: async () => graphQLContext?.GetMe(),
  });

  return { data, ...rest };
};
