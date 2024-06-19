import { useQuery } from "@tanstack/react-query";

import { useGraphQLContext } from "@flow/lib/gql";
import { Me } from "@flow/types/user";

import { GetMeQuery } from "../__gen__/graphql";

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

  // Not using react-query because no observers are needed on this
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
