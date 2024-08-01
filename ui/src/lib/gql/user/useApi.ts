import { GetMe, SearchUser, UpdateMe } from "@flow/types/user";

import { UpdateMeInput } from "../__gen__/graphql";

import { useQueries } from "./useQueries";

export enum UserQueryKeys {
  GetMe = "getMe",
  SearchUser = "User",
}

export const useUser = () => {
  const { useGetMeQuery, searchUserQuery, updateMeMutation } = useQueries();

  const useGetMe = (): GetMe => {
    const { data, ...rest } = useGetMeQuery();
    return {
      me: data,
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
    } catch (err) {
      return { me: undefined, ...rest };
    }
  };

  return {
    useGetMe,
    searchUser,
    updateMe,
  };
};
