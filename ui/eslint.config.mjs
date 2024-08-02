import { fixupPluginRules } from "@eslint/compat";
import * as graphql from "@graphql-eslint/eslint-plugin";
import config from "eslint-config-reearth";
import react from "eslint-plugin-react";
import reactHooks from "eslint-plugin-react-hooks";
import storybook from "eslint-plugin-storybook";
import tailwind from "eslint-plugin-tailwindcss";

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
      "@typescript-eslint/consistent-type-assertions": "off",
      "@graphql-eslint/executable-definitions": "error",
      "@graphql-eslint/fields-on-correct-type": "error",
      "@graphql-eslint/fragments-on-composite-type": "error",
      "@graphql-eslint/known-argument-names": "error",
      "@graphql-eslint/known-directives": "error",
      "@graphql-eslint/known-fragment-names": "error",
      "@graphql-eslint/known-type-names": "error",
      "@graphql-eslint/lone-anonymous-operation": "error",
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
      "@graphql-eslint/no-anonymous-operations": "error",
      "@graphql-eslint/no-deprecated": "error",
      "@graphql-eslint/no-duplicate-fields": "error",
      "@graphql-eslint/no-fragment-cycles": "error",
      "@graphql-eslint/no-undefined-variables": "error",
      "@graphql-eslint/one-field-subscriptions": "error",
      "@graphql-eslint/overlapping-fields-can-be-merged": "error",
      "@graphql-eslint/possible-fragment-spread": "error",
      "@graphql-eslint/provided-required-arguments": "error",
      "@graphql-eslint/scalar-leafs": "error",
      "@graphql-eslint/value-literals-of-correct-type": "error",
      "@graphql-eslint/variables-are-input-types": "error",
      "@graphql-eslint/variables-in-allowed-position": "error",
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
    ],
  },
];
