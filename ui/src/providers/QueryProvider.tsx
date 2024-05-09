import { QueryClient, QueryClientProvider as TanStackQueryProvider } from "@tanstack/react-query";

const queryClient = new QueryClient();

const QueryClientProvider = ({ children }: { children?: React.ReactNode }) => {
  return <TanStackQueryProvider client={queryClient}>{children}</TanStackQueryProvider>;
};

export { QueryClientProvider };
