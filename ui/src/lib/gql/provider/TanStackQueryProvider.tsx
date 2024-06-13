import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

export { useGraphQLContext } from "./GraphQLRequestProvider";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      // 20 seconds
      staleTime: 20 * 1000,
    },
  },
});

const TanStackQueryProvider = ({ children }: { children?: React.ReactNode }) => {
  return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
};

export { TanStackQueryProvider };
