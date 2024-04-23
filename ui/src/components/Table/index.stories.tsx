import type { Meta, StoryObj } from "@storybook/react";

import { DataTable } from ".";

const meta = {
  component: DataTable,
  parameters: {
    layout: "centered",
  },
  tags: ["autodocs"],
  argTypes: {},
} satisfies Meta<typeof DataTable>;

export default meta;
type Story = StoryObj<typeof meta>;

const commonArgs = {
  columns: [
    {
      accessorKey: "status",
      header: "Status",
    },
    {
      accessorKey: "email",
      header: "Email",
    },
    {
      accessorKey: "amount",
      header: "Amount",
    },
  ],
  data: [
    {
      id: "728ed52f",
      amount: 100,
      status: "pending",
      email: "m@example.com",
    },
    {
      id: "489e1d42",
      amount: 125,
      status: "processing",
      email: "example@gmail.com",
    },
  ],
};

export const Table: Story = {
  args: {
    ...commonArgs,
  },
};
