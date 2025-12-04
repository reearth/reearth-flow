import type { Meta, StoryObj } from "@storybook/react-vite";

import { Input } from ".";

const meta = {
  component: Input,
  parameters: {
    layout: "centered",
  },
  tags: ["autodocs"],
  argTypes: {},
} satisfies Meta<typeof Input>;

export default meta;
type Story = StoryObj<typeof meta>;

const commonArgs = {};

export const Default: Story = {
  args: {
    ...commonArgs,
  },
};
