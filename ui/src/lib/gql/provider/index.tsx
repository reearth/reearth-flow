import { GraphQLRequestProvider } from "./GraphQLRequestProvider";
import { GraphQLSubscriptionProvider } from "./GraphQLSubscriptionProvider";
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
        <GraphQLSubscriptionProvider accessToken={gqlAccessToken}>
          {children}
        </GraphQLSubscriptionProvider>
      </GraphQLRequestProvider>
    </TanStackQueryProvider>
  );
};

export { GraphQLProvider };
