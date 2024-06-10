import { useQuery } from "@tanstack/react-query";

import { useGraphQLContext } from "@flow/lib/gql";
import { GetMe } from "@flow/types/user";

export enum UserQueryKeys {
  GetMe = "getMe",
}

export const useUser = () => {
  const graphQLContext = useGraphQLContext();

  const getMeQuery = useQuery({
    queryKey: [UserQueryKeys.GetMe],
    queryFn: () => graphQLContext?.GetMe(),
    select: data => data?.me,
    staleTime: Infinity,
  });

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
