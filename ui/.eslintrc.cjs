module.exports = {
  extends: [
    "reearth",
    "plugin:storybook/recommended",
    "plugin:@tanstack/eslint-plugin-query/recommended",
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
};
