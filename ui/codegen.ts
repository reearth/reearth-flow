import { CodegenConfig } from "@graphql-codegen/cli";

const rootGQLDirectory = "src/lib/gql/__gen__";

const pluginsDirectory = `${rootGQLDirectory}/plugins`;

const config: CodegenConfig = {
  schema: "../api/gql/*.graphql",
  documents: ["src/**/*.tsx", "src/**/*.ts"],
  ignoreNoDocuments: true, // for better experience with the watcher
  generates: {
    [`${rootGQLDirectory}/`]: {
      preset: "client",
    },
    [`${pluginsDirectory}/generates.ts`]: {
      plugins: ["typescript", "typescript-operations", "typescript-react-query"],
      config: {
        fetcher: "graphql-request",
      },
    },
    [`${pluginsDirectory}/graphqlRequest.ts`]: {
      plugins: ["typescript", "typescript-operations", "typescript-graphql-request"],
      config: {
        rawRequest: true,
      },
    },
    // [`${pluginsDirectory}/namedOperationsObject.ts`]: {
    //   plugins: ["typescript", "named-operations-object"],
    // },
  },
};

export default config;
