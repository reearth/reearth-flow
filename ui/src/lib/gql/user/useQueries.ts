import { useQuery } from "@tanstack/react-query";

import { useGraphQLContext } from "@flow/lib/gql";

import { UserQueryKeys } from "./useApi";

export const useQueries = () => {
  const graphQLContext = useGraphQLContext();

  const useGetMeQuery = () =>
    useQuery({
      queryKey: [UserQueryKeys.GetMe],
      queryFn: async () => {
        const data = await graphQLContext?.GetMe();
        if (!data?.me) return;
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
    useGetMeQuery,
    searchUserQuery,
  };
};
