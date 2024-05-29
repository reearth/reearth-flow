import { CodegenConfig } from "@graphql-codegen/cli";

const rootGQLDirectory = "src/lib/gql/__gen__/";

const config: CodegenConfig = {
  schema: "../api/gql/*.graphql",
  documents: ["src/**/*.tsx", "src/**/*.ts"],
  ignoreNoDocuments: true, // for better experience with the watcher
  generates: {
    [rootGQLDirectory]: {
      preset: "client",
      plugins: [],
    },
  },
};

export default config;
