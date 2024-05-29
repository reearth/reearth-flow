import { useQuery } from "@tanstack/react-query";
import { useContext } from "react";

import { GraphQlSdkContext } from "@flow/providers";

// TODO: a lot of this stuff can be refactored to some other place
export const useMeQuery = () => {
  const sdk = useContext(GraphQlSdkContext);
  const { data, ...rest } = useQuery({
    // TODO: Can we autogenerate the key?
    queryKey: ["getMe"],
    queryFn: async () => sdk.GetMe(),
  });

  return { data, ...rest };
};
