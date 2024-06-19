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

  const useSearchUserQuery = (email: string) =>
    useQuery({
      queryKey: [UserQueryKeys.SearchUser, email],
      queryFn: () => graphQLContext?.SearchUser({ email }),
      select: (data: SearchUserQuery | undefined): User | undefined => {
        if (!data?.searchUser) return undefined;
        const user = data.searchUser;
        return {
          id: user.id,
          name: user.name,
          email: user.email,
        };
      },
      staleTime: Infinity,
    });

  return {
    getMeQuery,
    useSearchUserQuery,
  };
};
