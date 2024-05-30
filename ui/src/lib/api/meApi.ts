import { useQuery } from "@tanstack/react-query";

import { useGraphQLContext } from "@flow/lib/gql";

// TODO: a lot of this stuff can be refactored to some other place
export const useMeQuery = () => {
  const graphQLContext = useGraphQLContext();
  const { data, ...rest } = useQuery({
    // TODO: Can we autogenerate the key?
    queryKey: ["getMe"],
    queryFn: async () => graphQLContext?.GetMe(),
  });

  return { data, ...rest };
};
