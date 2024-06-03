import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

export { useGraphQLContext } from "./GraphQLRequestProvider";

const queryClient = new QueryClient();

const TanStackQueryProvider = ({ children }: { children?: React.ReactNode }) => {
  return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
};

export { TanStackQueryProvider };
