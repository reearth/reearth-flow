import { useQuery } from "@tanstack/react-query";
import { useContext } from "react";

import { GET_ME } from "@flow/lib/gql";
import { GraphQlContext } from "@flow/providers";

export const useMeQuery = () => {
  const client = useContext(GraphQlContext);
  const { data, ...rest } = useQuery({
    queryKey: ["getMe"],
    queryFn: async () => client.request(GET_ME),
  });

  return { data, ...rest };
};
