import {
  GetMe,
  GetMeAndWorkspaces,
  SearchUser,
  UpdateMe,
} from "@flow/types/user";

import { UpdateMeInput } from "../__gen__/graphql";

import { useQueries } from "./useQueries";

export enum UserQueryKeys {
  GetMe = "getMe",
  GetMeAndWorkspaces = "getMeAndWorkspaces",
  SearchUser = "User",
}

export const useUser = () => {
  const {
    useGetMeQuery,
    useGetMeAndWorkspacesQuery,
    searchUserQuery,
    updateMeMutation,
  } = useQueries();

  const useGetMe = (): GetMe => {
    const { data, ...rest } = useGetMeQuery();
    return {
      me: data,
      ...rest,
    };
  };

  const useGetMeAndWorkspaces = (): GetMeAndWorkspaces => {
    const { data, ...rest } = useGetMeAndWorkspacesQuery();
    return {
      me: data,
      workspaces: data?.workspaces?.map((workspace) => ({
        id: workspace.id,
        name: workspace.name,
        personal: workspace.personal,
        members: workspace.members.map((member) => ({
          userId: member.userId,
          role: member.role,
          user: member.user
            ? {
                id: member.user.id,
                email: member.user.email,
                name: member.user.name,
              }
            : undefined,
        })),
      })),
      ...rest,
    };
  };

  const searchUser = async (email: string): Promise<SearchUser> => {
    const data = await searchUserQuery(email);
    return {
      user: data,
    };
  };

  const updateMe = async (input: UpdateMeInput): Promise<UpdateMe> => {
    const { mutateAsync, ...rest } = updateMeMutation;
    try {
      const me = await mutateAsync(input);
      return { me, ...rest };
    } catch (_err) {
      return { me: undefined, ...rest };
    }
  };

  return {
    useGetMe,
    useGetMeAndWorkspaces,
    searchUser,
    updateMe,
  };
};
