import { RJSFSchema } from "@rjsf/utils";
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

const commonArgs: {
  schema: RJSFSchema;
} = {
  schema: {
    title: "A registration form",
    description: "A simple form example.",
    type: "object",
    required: ["firstName", "lastName"],
    properties: {
      firstName: {
        type: "string",
        title: "First name",
        default: "Chuck",
      },
      lastName: {
        type: "string",
        title: "Last name",
      },
      age: {
        type: "integer",
        title: "Age",
      },
      bio: {
        type: "string",
        title: "Bio",
      },
      password: {
        type: "string",
        title: "Password",
        minLength: 3,
      },
      telephone: {
        type: "string",
        title: "Telephone",
        minLength: 10,
      },
    },
  },
};

export const Default: Story = {
  args: {
    ...commonArgs,
  },
};
