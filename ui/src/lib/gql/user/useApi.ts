import { GetMe, SearchUser } from "@flow/types/user";

import { useQueries } from "./useQueries";

export enum UserQueryKeys {
  GetMe = "getMe",
  SearchUser = "User",
}

export const useUser = () => {
  const { getMeQuery, searchUserQuery } = useQueries();

  const getMe = (): GetMe => {
    const { data, ...rest } = getMeQuery;
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
    getMe,
    searchUser,
  };
};
