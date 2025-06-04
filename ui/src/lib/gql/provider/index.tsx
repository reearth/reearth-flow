import { DocumentGraphQLProvider } from "./DocumentGraphQLProvider";
import { GraphQLRequestProvider } from "./GraphQLRequestProvider";
import { TanStackQueryProvider } from "./TanStackQueryProvider";

export { useGraphQLContext } from "./GraphQLRequestProvider";
export { useDocumentGraphQLContext } from "./DocumentGraphQLProvider";

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
        <DocumentGraphQLProvider accesstoken={gqlAccessToken}>
          {children}
        </DocumentGraphQLProvider>
      </GraphQLRequestProvider>
    </TanStackQueryProvider>
  );
};

export { GraphQLProvider };
