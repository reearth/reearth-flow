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
    queryFn: async () => {
      if (!graphQLContext?.GetMe) return;
      const data = await graphQLContext?.GetMe();
      return data?.me;
    },
    staleTime: Infinity,
  });

  const getMe = (): GetMe => {
    const { data: me, ...rest } = getMeQuery;
    return {
      me,
      ...rest,
    };
  };

  return {
    getMe,
  };
};
