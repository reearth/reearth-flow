import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { useGraphQLContext } from "@flow/lib/gql";

import { UpdateMeInput } from "../__gen__/graphql";

import { UserQueryKeys } from "./useApi";

export const useQueries = () => {
  const graphQLContext = useGraphQLContext();
  const queryClient = useQueryClient();

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
          lang: me.lang,
        };
      },
      staleTime: Infinity,
    });

  const useGetMeAndWorkspacesQuery = () =>
    useQuery({
      queryKey: [UserQueryKeys.GetMeAndWorkspaces],
      queryFn: async () => {
        const data = await graphQLContext?.GetMeAndWorkspaces();
        if (!data?.me) return;
        const me = data.me;
        return {
          id: me.id,
          name: me.name,
          email: me.email,
          myWorkspaceId: me.myWorkspaceId,
          lang: me.lang,
          workspaces: me.workspaces,
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
    } catch (_err) {
      return;
    }
  };

  const updateMeMutation = useMutation({
    mutationFn: async (input: UpdateMeInput) => {
      const data = await graphQLContext?.UpdateMe({ input });
      if (data?.updateMe?.me) {
        const { id, name, email, lang } = data.updateMe.me;
        return {
          id,
          name,
          email,
          lang,
        };
      }
    },
    onSuccess: () =>
      queryClient.invalidateQueries({
        queryKey: [UserQueryKeys.GetMe],
      }),
  });

  return {
    useGetMeQuery,
    useGetMeAndWorkspacesQuery,
    searchUserQuery,
    updateMeMutation,
  };
};
