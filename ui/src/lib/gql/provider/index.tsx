import { GraphQLRequestProvider } from "./GraphQLRequestProvider";
import { TanStackQueryProvider } from "./TanStackQueryProvider";

export { useGraphQLContext } from "./GraphQLRequestProvider";

const GraphQLProvider = ({
  gqlAccessToken,
  children,
}: {
  gqlAccessToken?: string;
  children?: React.ReactNode;
}) => {
  return (
    <TanStackQueryProvider>
      <GraphQLRequestProvider accesstoken={gqlAccessToken}>
        {children}
      </GraphQLRequestProvider>
    </TanStackQueryProvider>
  );
};

export { GraphQLProvider };
