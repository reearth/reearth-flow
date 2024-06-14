import { CodegenConfig } from "@graphql-codegen/cli";

const rootGQLDirectory = "src/lib/gql/__gen__/";
const pluginsDirectory = `${rootGQLDirectory}/plugins`;

const config: CodegenConfig = {
  schema: "../api/gql/*.graphql",
  documents: ["src/lib/gql/**/queries.graphql"],
  ignoreNoDocuments: true, // for better experience with the watcher
  generates: {
    [rootGQLDirectory]: {
      preset: "client",
    },
    [`${pluginsDirectory}/graphql-request.ts`]: {
      plugins: ["typescript", "typescript-operations", "typescript-graphql-request"],
    },
  },
};

export default config;
