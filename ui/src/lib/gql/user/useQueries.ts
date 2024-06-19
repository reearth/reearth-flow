import { useQuery } from "@tanstack/react-query";

import { useGraphQLContext } from "@flow/lib/gql";
import { Me, User } from "@flow/types/user";

import { GetMeQuery, SearchUserQuery } from "../__gen__/graphql";

import { UserQueryKeys } from "./useApi";

export const useQueries = () => {
  const graphQLContext = useGraphQLContext();

  const getMeQuery = useQuery({
    queryKey: [UserQueryKeys.GetMe],
    queryFn: () => graphQLContext?.GetMe(),
    select: (data: GetMeQuery | undefined): Me | undefined => {
      if (!data?.me) return undefined;
      const me = data.me;
      return {
        id: me.id,
        name: me.name,
        email: me.email,
        myWorkspaceId: me.myWorkspaceId,
      };
    },
    staleTime: Infinity,
  });

  // Not using react-query because it was returning a hook and a function was needed
  const searchUserQuery = async (email: string) => {
    try {
      const data = await graphQLContext?.SearchUser({ email });
      if (!data?.searchUser) return;
      return {
        id: data.searchUser.id,
        name: data.searchUser.name,
        email: data.searchUser.email,
      };
    } catch (err) {
      return;
    }
  };

  return {
    getMeQuery,
    searchUserQuery,
  };
};
