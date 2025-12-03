import type { Meta, StoryObj } from "@storybook/react-vite";

import Loading from "./Splashscreen";

const meta = {
  component: Loading,
  parameters: {
    layout: "centered",
  },
  tags: ["autodocs"],
  argTypes: {},
} satisfies Meta<typeof Loading>;

export default meta;
type Story = StoryObj<typeof meta>;

const commonArgs = {};

export const Default: Story = {
  args: {
    ...commonArgs,
  },
};
