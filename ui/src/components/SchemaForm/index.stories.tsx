import type { Meta, StoryObj } from "@storybook/react";

import { SchemaForm } from ".";

const meta = {
  component: SchemaForm,
  parameters: {
    layout: "centered",
  },
  tags: ["autodocs"],
  argTypes: {},
} satisfies Meta<typeof SchemaForm>;

export default meta;
type Story = StoryObj<typeof meta>;

const commonArgs = {};

export const Default: Story = {
  args: {
    ...commonArgs,
  },
};
