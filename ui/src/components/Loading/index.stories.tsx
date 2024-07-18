import type { Meta, StoryObj } from "@storybook/react";

import { Loading } from ".";

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
