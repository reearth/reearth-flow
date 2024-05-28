import { useQuery } from "@tanstack/react-query";

import { GET_ME, useClient } from "@flow/lib/gql";

export const useMeQuery = () => {
  const client = useClient();
  const { data, ...rest } = useQuery({
    queryKey: ["getMe"],
    queryFn: async () => client.request(GET_ME),
  });

  return { data, ...rest };
};
