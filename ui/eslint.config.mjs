import * as graphql from "@graphql-eslint/eslint-plugin";
import config from "eslint-config-reearth";
import storybook from "eslint-plugin-storybook";
import tailwind from "eslint-plugin-tailwindcss";

const storyBookConfig = {
  files: ["*.stories.@(ts|tsx|js|jsx|mjs|cjs)"],
  plugins: {
    storybook,
  },
};

const customTailwindConfig = [
  ...tailwind.configs["flat/recommended"],
  {
    rules: {
      "tailwindcss/no-custom-classname": [
        "warn",
        {
          whitelist: ["nopan", "nodrag", "nowheel", "destructive", "dndnode-"],
        },
      ],
    },
  },
];

const graphqlConfig = {
  files: ["**/*.graphql"],
  plugins: {
    "@graphql-eslint": { rules: graphql.rules },
  },
  languageOptions: {
    parser: graphql.parser,
    parserOptions: {
      graphQLConfig: {
        skipGraphQLConfig: true,
        schema: "../api/gql/*.graphql",
      },
    },
  },
  rules: {
    ...graphql.flatConfigs["operations-recommended"].rules,
    "@typescript-eslint/consistent-type-assertions": "off",
    "@graphql-eslint/require-selections": "off",
    "@graphql-eslint/no-unused-fragments": "off",
    "@graphql-eslint/unique-fragment-name": "off",
    "@graphql-eslint/unique-operation-name": "off",
    "@graphql-eslint/selection-set-depth": "off",
    "@graphql-eslint/known-fragment-names": "off",
    "@graphql-eslint/no-undefined-variables": "off",
    "@graphql-eslint/no-unused-variables": "off",
    "@graphql-eslint/naming-convention": [
      "error",
      {
        VariableDefinition: "camelCase",
        OperationDefinition: {
          style: "PascalCase",
          forbiddenPrefixes: ["Query", "Mutation"],
          forbiddenSuffixes: ["Query", "Mutation"],
        },
        FragmentDefinition: {
          style: "PascalCase",
          forbiddenPrefixes: ["Fragment"],
          forbiddenSuffixes: ["Fragment"],
        },
      },
    ],
  },
};

/** @type { import("eslint").Linter.Config[] } */
export default [
  ...config("flow"),
  ...customTailwindConfig,
  storyBookConfig,
  graphqlConfig,
  {
    ignores: ["coverage/*", "src/lib/gql/__gen__", "src/routeTree.gen.ts"],
  },
];
