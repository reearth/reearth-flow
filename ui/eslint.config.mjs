import * as graphql from "@graphql-eslint/eslint-plugin";
import config from "eslint-config-reearth";
import storybook from "eslint-plugin-storybook";

const storyBookConfig = {
  files: ["*.stories.@(ts|tsx|js|jsx|mjs|cjs)"],
  plugins: {
    storybook,
  },
};

// const customTailwindConfig = [
//   ...tailwind.configs["flat/recommended"],
//   {
//     rules: {
//       "tailwindcss/no-custom-classname": [
//         "warn",
//         {
//           whitelist: [
//             "nopan",
//             "nodrag",
//             "nowheel",
//             "destructive",
//             "warning",
//             "dndnode-",
//           ],
//         },
//       ],
//     },
//   },
// ];

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
        schema: "../server/api/gql/*.graphql",
      },
    },
  },
  rules: {
    ...graphql.configs["flat/operations-recommended"].rules,
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

const flowConfig = {
  rules: {
    "@typescript-eslint/no-explicit-any": "off", // Eventually we want to turn this back on, but for now its just a headache @KaWaite
  },
};

/** @type { import("eslint").Linter.Config[] } */
export default [
  ...config("flow"),
  flowConfig,
  storyBookConfig,
  graphqlConfig,
  {
    ignores: ["coverage/*", "src/lib/gql/__gen__", "src/routeTree.gen.ts"],
  },
];
