import type { Decorator, Meta, StoryObj } from "@storybook/react-vite";
import { useEffect, useState } from "react";

import type { Config } from "@flow/config";

import { EcosystemNavigator } from ".";

const THEMES = ["light", "dark", "midnight"] as const;

type Theme = (typeof THEMES)[number];

// The popover renders in a portal on <body>, so the theme has to be set on
// the document root (as ThemeProvider does) rather than on a wrapper. Storybook
// reuses the preview document across stories, so the previous theme is
// restored on unmount.
const ThemeRoot: React.FC<{ theme: Theme; children: React.ReactNode }> = ({
  theme,
  children,
}) => {
  useEffect(() => {
    const root = document.documentElement;
    const previousClasses = THEMES.filter((t) => root.classList.contains(t));
    const previousDataTheme = root.getAttribute("data-theme");
    root.classList.remove(...THEMES);
    root.classList.add(theme);
    root.setAttribute("data-theme", theme);
    return () => {
      root.classList.remove(...THEMES);
      root.classList.add(...previousClasses);
      if (previousDataTheme === null) {
        root.removeAttribute("data-theme");
      } else {
        root.setAttribute("data-theme", previousDataTheme);
      }
    };
  }, [theme]);
  return (
    <div className="flex h-[620px] w-[400px] items-start bg-background p-4 text-foreground">
      {children}
    </div>
  );
};

// Storybook has no reearth_config.json, so stand in for the link URLs.
const PLACEHOLDER_URLS: Config = {
  reearthHomeUrl: "#",
  reearthDashboardUrl: "#",
  reearthVisualizerUrl: "#",
  reearthCmsUrl: "#",
  navaraUrl: "#",
  reearthTerrainUrl: "#",
  reearthBuildingsUrl: "#",
  reearthPapersUrl: "#",
  reearthCommunityUrl: "#",
};

const NO_URLS: Config = {};

// The config has to be in place before the story's first render, so it is set
// during render as well as in the effect (which re-applies it if the effect is
// re-run), and the previous config is restored on unmount.
const ConfigRoot: React.FC<{ config: Config; children: React.ReactNode }> = ({
  config,
  children,
}) => {
  const [previousConfig] = useState(() => {
    const previous = window.REEARTH_CONFIG;
    window.REEARTH_CONFIG = config;
    return previous;
  });
  useEffect(() => {
    window.REEARTH_CONFIG = config;
    return () => {
      window.REEARTH_CONFIG = previousConfig;
    };
  }, [config, previousConfig]);
  return children;
};

const withConfig: Decorator = (Story, { parameters }) => (
  <ConfigRoot config={parameters.noUrls ? NO_URLS : PLACEHOLDER_URLS}>
    <Story />
  </ConfigRoot>
);

const withTheme: Decorator = (Story, { parameters }) => (
  <ThemeRoot theme={parameters.theme ?? "dark"}>
    <Story />
  </ThemeRoot>
);

const meta = {
  component: EcosystemNavigator,
  parameters: {
    layout: "centered",
  },
  tags: ["autodocs"],
  argTypes: {
    side: {
      control: "select",
      options: ["top", "bottom", "left", "right"],
    },
    align: {
      control: "select",
      options: ["start", "center", "end"],
    },
  },
  decorators: [withConfig, withTheme],
} satisfies Meta<typeof EcosystemNavigator>;

export default meta;
type Story = StoryObj<typeof meta>;

const openOnLoad: Story["play"] = async ({ canvas, userEvent }) => {
  await userEvent.click(canvas.getByRole("button"));
};

export const Default: Story = {
  args: {},
};

export const Open: Story = {
  args: {},
  play: openOnLoad,
};

export const Editor: Story = {
  args: {
    iconSize: 16,
  },
  play: openOnLoad,
};

export const Light: Story = {
  args: {},
  parameters: { theme: "light" },
  play: openOnLoad,
};

export const Midnight: Story = {
  args: {},
  parameters: { theme: "midnight" },
  play: openOnLoad,
};

export const NoUrls: Story = {
  args: {},
  parameters: { noUrls: true },
  play: openOnLoad,
};
