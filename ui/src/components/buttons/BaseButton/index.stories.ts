import type { Meta, StoryObj } from "@storybook/react-vite";

import { Button } from ".";

const meta = {
  //   title: "Example/Button",
  component: Button,
  parameters: {
    // Optional parameter to center the component in the Canvas. More info: https://storybook.js.org/docs/configure/story-layout
    layout: "centered",
  },
  // This component will have an automatically generated Autodocs entry: https://storybook.js.org/docs/writing-docs/autodocs
  tags: ["autodocs"],
  // More on argTypes: https://storybook.js.org/docs/api/argtypes
  argTypes: {},
} satisfies Meta<typeof Button>;

export default meta;
type Story = StoryObj<typeof meta>;

const commonArgs = {
  children: "Button",
};

export const Default: Story = {
  args: {
    size: "default",
    ...commonArgs,
  },
};

export const Large: Story = {
  args: {
    size: "lg",
    ...commonArgs,
  },
};

export const Small: Story = {
  args: {
    size: "sm",
    ...commonArgs,
  },
};

export const Icon: Story = {
  args: {
    size: "icon",
    children: "X",
  },
};

export const Outline: Story = {
  args: {
    size: "sm",
    variant: "outline",
    ...commonArgs,
  },
};

export const Link: Story = {
  args: {
    size: "sm",
    variant: "link",
    ...commonArgs,
  },
};
