import { fixupPluginRules } from "@eslint/compat";
import * as graphql from "@graphql-eslint/eslint-plugin";
import config from "eslint-config-reearth";
import react from "eslint-plugin-react";
import reactHooks from "eslint-plugin-react-hooks";
import storybook from "eslint-plugin-storybook";
import tailwind from "eslint-plugin-tailwindcss";

/** @type { import("eslint").Linter.Config[] } */

export default [
  ...config,
  ...tailwind.configs["flat/recommended"],
  {
    files: ["**/*.{js,jsx,ts,tsx}"],
    plugins: {
      react,
      "react-hooks": fixupPluginRules(reactHooks),
    },
    settings: {
      react: {
        version: "detect",
      },
    },
    rules: {
      "react/prop-types": "off",
      "react-hooks/rules-of-hooks": "error",
      "react-hooks/exhaustive-deps": "error",
      "react/jsx-no-useless-fragment": [
        "warn",
        {
          allowExpressions: true,
        },
      ],
      "react/self-closing-comp": [
        "warn",
        {
          component: true,
          html: true,
        },
      ],
    },
  },
  {
    rules: {
      "tailwindcss/no-custom-classname": [
        "warn",
        {
          whitelist: ["nopan", "nodrag", "nowheel", "dndnode-"],
        },
      ],
    },
  },
  {
    files: ["*.stories.@(ts|tsx|js|jsx|mjs|cjs)"],
    plugins: {
      storybook,
    },
  },
  {
    rules: {
      "import/order": [
        "warn",
        {
          pathGroups: [
            {
              pattern: "@flow/**",
              group: "external",
              position: "after",
            },
          ],
          pathGroupsExcludedImportTypes: ["builtin"],
          "newlines-between": "always",
          alphabetize: {
            order: "asc",
            caseInsensitive: true,
          },
        },
      ],

      "@typescript-eslint/no-unused-vars": [
        "warn",
        {
          args: "all",
          argsIgnorePattern: "^_",
          caughtErrors: "all",
          caughtErrorsIgnorePattern: "^_",
          destructuredArrayIgnorePattern: "^_",
          varsIgnorePattern: "^_",
          ignoreRestSiblings: true,
        },
      ],
      "@typescript-eslint/no-invalid-void-type": "warn",
      "@typescript-eslint/array-type": "warn",
      "@typescript-eslint/consistent-indexed-object-style": "warn",
      "node/no-unsupported-features/es-syntax": [
        "error",
        {
          version: ">=20.13.0",
          ignores: ["dynamicImport", "modules"],
        },
      ],
      "@typescript-eslint/no-explicit-any": "off",
      "@typescript-eslint/consistent-type-definitions": "off",
      "@typescript-eslint/no-empty-function": "off",
      "node/no-extraneous-import": "off",
    },
  },
  {
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
  },
  {
    ignores: [
      "build/*",
      "dist/*",
      "coverage/*",
      "node_modules/*",
      "storybook-static/*",
      "!.storybook/",
      ".storybook/public/*",
      "src/lib/gql/__gen__",
      "src/routeTree.gen.ts",
    ],
  },
];
