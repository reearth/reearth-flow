import { GraphQLRequestProvider } from "./GraphQLRequestProvider";
import { TanStackQueryProvider } from "./TanStackQueryProvider";

export { useGraphQLContext } from "./GraphQLRequestProvider";

const GraphQLProvider = ({ children }: { children?: React.ReactNode }) => {
  return (
    <TanStackQueryProvider>
      <GraphQLRequestProvider>{children}</GraphQLRequestProvider>
    </TanStackQueryProvider>
  );
};

export { GraphQLProvider };
