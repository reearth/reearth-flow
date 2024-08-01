import * as graphql from '@graphql-eslint/eslint-plugin';
import config from "eslint-config-reearth";
import storybook from "eslint-plugin-storybook";
import tailwind from "eslint-plugin-tailwindcss";

export default [
    ...config,
    ...tailwind.configs["flat/recommended"],
    {
        files: ["*.stories.@(ts|tsx|js|jsx|mjs|cjs)"],
        plugins: {
            storybook: storybook,
        },
    },
    {
        plugins: {
            "@graphql-eslint": { rules: graphql.rules },
        },
        languageOptions: {
            parserOptions: {
                graphQLConfig: {
                    skipGraphQLConfig: true,
                    schema: "../api/gql/*.graphql",
                    operations: "src/lib/gql/**/*.graphql",
                }
            }
        }
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
            "node/no-extraneous-import": "off",
            "@typescript-eslint/no-unused-vars": ["warn", {
                "args": "all",
                "argsIgnorePattern": "^_",
                "caughtErrors": "all",
                "caughtErrorsIgnorePattern": "^_",
                "destructuredArrayIgnorePattern": "^_",
                "varsIgnorePattern": "^_",
                "ignoreRestSiblings": true
            }],
            "@typescript-eslint/no-invalid-void-type": "warn",
            "@typescript-eslint/array-type": "warn",
            "@typescript-eslint/consistent-indexed-object-style": "warn",
            "node/no-unsupported-features/es-syntax": ["error", {
                "version": ">=20.13.0",
                "ignores": ["dynamicImport", "modules"]
            }],
            'tailwindcss/no-custom-classname': ['warn', {
                whitelist: ["nopan", "nodrag", "nowheel", "dndnode-"]
            }],
            "@graphql-eslint/naming-convention": [
                "error",
                {
                    VariableDefinition: "camelCase",
                    FieldDefinition: "camelCase",
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
                }
            ],
            "@typescript-eslint/no-explicit-any": "off",
            "@typescript-eslint/consistent-type-definitions": "off",
            "@typescript-eslint/no-empty-function": "off",
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