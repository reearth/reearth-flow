import { useQuery } from "@tanstack/react-query";
import request from "graphql-request";

import { config } from "@flow/config";
import { GET_ME } from "@flow/lib/gql";

export const useMeQuery = () => {
  const { api } = config();
  const { data, ...rest } = useQuery({
    queryKey: ["getMe"],
    queryFn: async () => request(`${api}/graphql`, GET_ME),
  });

  console.log("DATA USER: ", data);
  return { data, ...rest };
};
