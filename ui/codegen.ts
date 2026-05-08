import { CodegenConfig } from "@graphql-codegen/cli";

const rootGQLDirectory = "src/lib/gql/__gen__/";
const pluginsDirectory = `${rootGQLDirectory}/plugins`;

const scalarsConfig = {
  scalars: {
    ID: { input: "string", output: "string" },
    Any: { input: "any", output: "any" },
    Bytes: { input: "any", output: "any" },
    DateTime: { input: "any", output: "any" },
    FileSize: { input: "any", output: "any" },
    JSON: { input: "any", output: "any" },
    Lang: { input: "any", output: "any" },
    URL: { input: "any", output: "any" },
    Upload: { input: "any", output: "any" },
  },
};

const config: CodegenConfig = {
  schema: "../server/api/gql/*.graphql",
  documents: ["src/lib/gql/**/*.graphql"],
  ignoreNoDocuments: true, // for better experience with the watcher
  generates: {
    [rootGQLDirectory]: {
      preset: "client",
      config: scalarsConfig,
    },
    [`${pluginsDirectory}/graphql-request.ts`]: {
      plugins: ["typescript-operations", "typescript-graphql-request"],
      config: scalarsConfig,
    },
  },
};

export default config;
