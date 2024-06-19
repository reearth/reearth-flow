import { GetMe, SearchUser } from "@flow/types/user";

import { useQueries } from "./useQueries";

export enum UserQueryKeys {
  GetMe = "getMe",
  SearchUser = "User",
}

export const useUser = () => {
  const { getMeQuery, useSearchUserQuery } = useQueries();

  const getMe = (): GetMe => {
    const { data, ...rest } = getMeQuery;
    return {
      me: data,
      ...rest,
    };
  };

  const useSearchUser = (email: string): SearchUser => {
    const { data, ...rest } = useSearchUserQuery(email);
    return {
      user: data,
      ...rest,
    };
  };

  return {
    getMe,
    useSearchUser,
  };
};
