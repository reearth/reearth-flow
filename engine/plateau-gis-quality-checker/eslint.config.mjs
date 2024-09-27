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

/** @type { import("eslint").Linter.Config[] } */
export default [
  ...config("flow"),
  ...customTailwindConfig,
  storyBookConfig,
  {
    ignores: ["coverage/*", "src/routeTree.gen.ts"],
  },
];
