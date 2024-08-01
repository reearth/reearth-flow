import { GetMe, SearchUser, UpdateMe } from "@flow/types/user";

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

  const updateMe = async ({ name, email }: { name: string; email: string }): Promise<UpdateMe> => {
    const { mutateAsync, ...rest } = updateMeMutation;
    try {
      const me = await mutateAsync({ name, email });
      return { me, ...rest };
    } catch (_err) {
      return { me: undefined, ...rest };
    }
  };

  return {
    useGetMe,
    searchUser,
    updateMe,
  };
};
