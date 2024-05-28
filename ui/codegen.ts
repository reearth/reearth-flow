import { CodegenConfig } from "@graphql-codegen/cli";

const config: CodegenConfig = {
  schema: "../api/gql/*.graphql",
  documents: ["src/**/*.tsx", "src/**/*.ts"],
  ignoreNoDocuments: true, // for better experience with the watcher
  generates: {
    "./src/lib/gql/__gen__/": {
      preset: "client",
      // TODO: Add necessary plugins
      // plugins:
    },
  },
};

export default config;
