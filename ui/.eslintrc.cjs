module.exports = {
  extends: [
    "reearth",
    "plugin:storybook/recommended",
    "plugin:@tanstack/eslint-plugin-query/recommended",
    "plugin:tailwindcss/recommended",
  ],
  root: true,
  env: { browser: true, es2020: true },
  ignorePatterns: ["dist", ".eslintrc.cjs"],
  parser: "@typescript-eslint/parser",
  plugins: ["react-refresh"],
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
    "@tanstack/query/exhaustive-deps": 0,
  },
  overrides: [
    {
      files: ["*.graphql"],
      parser: "@graphql-eslint/eslint-plugin",
      plugins: ["@graphql-eslint"],
      extends: "plugin:@graphql-eslint/operations-recommended",
      rules: {
        "@graphql-eslint/naming-convention": [
          "error",
          {
            VariableDefinition: "camelCase",
            OperationDefinition: {
              style: "PascalCase",
              forbiddenPrefixes: ["Query", "Mutation", "Subscription"],
              forbiddenSuffixes: ["Query", "Mutation", "Subscription"],
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
  ],
  parserOptions: {
    skipGraphQLConfig: true,
    schema: "../api/gql/*.graphql",
    operations: "src/lib/gql/**/*.graphql",
  },
  settings: {
    tailwindcss: {
      // Mention extra CSS classes that are not part of tailwind here
      whitelist: ["nopan", "nodrag", "nowheel", "dndnode-"],
    },
  },
};
