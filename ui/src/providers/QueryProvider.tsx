import { QueryClient, QueryClientProvider as TanStackQueryProvider } from "@tanstack/react-query";
import { useEffect, useState } from "react";

import { useAuth } from "@flow/lib/auth";

const graphQLClient = new QueryClient();

const QueryClientProvider = ({ children }: { children?: React.ReactNode }) => {
  const { getAccessToken } = useAuth();
  const [token, setToken] = useState<string | undefined>();

  useEffect(() => {
    (async () => {
      const token = await getAccessToken();
      setToken(token);
    })();
    console.log("HI12344544");
  }, [getAccessToken]);

  // const graphQLClient = new QueryClient(`${api}/graphql`, defaultOptions: {
  //   headers: {
  //     authorization: `Bearer ${e2eAccessToken() || token}`,
  //   },
  // });
  return token ? (
    <TanStackQueryProvider client={graphQLClient}>{children}</TanStackQueryProvider>
  ) : (
    children
  );
};

export { QueryClientProvider };
