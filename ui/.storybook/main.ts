import type { StorybookConfig } from "@storybook/react-vite";
import type { PluginOption } from "vite";

const config: StorybookConfig = {
  stories: ["../src/**/*.mdx", "../src/**/*.stories.@(js|jsx|mjs|ts|tsx)"],

  core: {
    disableTelemetry: true,
  },

  staticDirs: ["./assets"],
  addons: ["@storybook/addon-links", "@storybook/addon-onboarding"],

  framework: {
    name: "@storybook/react-vite",
    options: {},
  },

  // Storybook inherits the project vite.config.ts, so TanStackRouterVite() runs
  // here too and regenerates src/routeTree.gen.ts. With `yarn start` also up,
  // both servers write that file and each trips the other's watcher — an endless
  // reload loop, and a vicious one when the two hold different generator
  // versions. Storybook never needs to generate the tree, so drop just the
  // generator; the code-splitter and HMR plugins still transform route files.
  viteFinal: async (config) => ({
    ...config,
    plugins: ((config.plugins ?? []) as unknown[])
      .flat(Infinity)
      .filter(
        (plugin) =>
          !(
            typeof plugin === "object" &&
            plugin !== null &&
            "name" in plugin &&
            plugin.name === "tanstack:router-generator"
          ),
      ) as PluginOption[],
  }),
};
export default config;
