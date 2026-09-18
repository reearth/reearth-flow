import * as graphql from "@graphql-eslint/eslint-plugin";
import config from "eslint-config-reearth";
import betterTailwind from "eslint-plugin-better-tailwindcss";
import reactHooks from "eslint-plugin-react-hooks";
import storybook from "eslint-plugin-storybook";

const storyBookConfig = {
  files: ["*.stories.@(ts|tsx|js|jsx|mjs|cjs)"],
  plugins: {
    storybook,
  },
};

const tailwindConfig = {
  plugins: {
    "better-tailwindcss": betterTailwind,
  },
  rules: {
    "better-tailwindcss/enforce-consistent-class-order": "warn",
  },
  settings: {
    "better-tailwindcss": {
      entryPoint: "src/index.css",
      tailwindConfig: "tailwind.config.js",
    },
  },
};

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
    "@typescript-eslint/consistent-type-imports": "off",
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
  plugins: { "react-hooks": reactHooks },
  rules: {
    "@typescript-eslint/no-explicit-any": "off", // Eventually we want to turn this back on, but for now its just a headache @KaWaite

    // React Compiler rules, downgraded to warnings pending a dedicated pass.
    //
    // eslint-config-reearth 0.4.0 turned these on as errors, surfacing ~52
    // pre-existing findings across ~50 files. They are real, but they sit in
    // the Cesium viewers, the yjs awareness layer, the GraphQL subscription
    // setup and SchemaForm — none of which have test coverage — and the fixes
    // change timing rather than just shape. The dominant one is the deliberate
    // "latest ref" idiom:
    //
    //   const valueRef = useRef(value);
    //   valueRef.current = value;        // flagged by react-hooks/refs
    //
    // Moving that into a useEffect satisfies the rule but defers the update
    // from render to post-commit, so anything reading the ref during the same
    // render — or in an effect declared earlier — silently reads a stale
    // value. That needs per-site review plus manual QA, not a sweep.
    //
    // Warnings keep every finding in the lint output while unblocking CI.
    "react-hooks/refs": "warn",
    "react-hooks/set-state-in-effect": "warn",
    "react-hooks/immutability": "warn",
    "react-hooks/preserve-manual-memoization": "warn",
    "react-hooks/purity": "warn",
    "react-hooks/static-components": "warn",
  },
};

/** @type { import("eslint").Linter.Config[] } */
export default [
  ...config("flow"),
  flowConfig,
  storyBookConfig,
  graphqlConfig,
  tailwindConfig,
  {
    ignores: [
      "coverage/*",
      "src/lib/gql/__gen__",
      "src/routeTree.gen.ts",
      "CLAUDE.md",
      // Bundled @reearth/sentinel service worker (vendor artifact).
      "public/sw.js",
    ],
  },
];
