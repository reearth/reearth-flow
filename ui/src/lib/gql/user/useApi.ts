import { GetMe, SearchUser } from "@flow/types/user";

import { useQueries } from "./useQueries";

export enum UserQueryKeys {
  GetMe = "getMe",
  SearchUser = "User",
}

export const useUser = () => {
  const { useGetMeQuery, searchUserQuery } = useQueries();

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

  return {
    useGetMe,
    searchUser,
  };
};
