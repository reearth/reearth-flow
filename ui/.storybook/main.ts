import type { StorybookConfig } from "@storybook/react-vite";

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
};
export default config;
