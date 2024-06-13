import { GetMe } from "@flow/types/user";

import { useQueries } from "./useQueries";

export enum UserQueryKeys {
  GetMe = "getMe",
}

export const useUser = () => {
  const { getMeQuery } = useQueries();

  const getMe = (): GetMe => {
    const { data, ...rest } = getMeQuery;
    return {
      me: data,
      ...rest,
    };
  };

  return {
    getMe,
  };
};
