module.exports = {
  env: { browser: true, es2020: true },
  extends: ["reearth", "plugin:storybook/recommended"],
  ignorePatterns: ["dist", ".eslintrc.cjs"],
  parser: "@typescript-eslint/parser",
  plugins: ["react-refresh"],
  // root: true,
  // rules: {
  //   "react-refresh/only-export-components": [
  //     "warn",
  //     { allowConstantExport: true },
  //   ],
  // },
}
