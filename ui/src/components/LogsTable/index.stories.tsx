import type { Meta, StoryObj } from "@storybook/react";

import { logData } from "./logsData";

import { LogsTable } from ".";

const meta = {
  component: LogsTable,
  parameters: {
    layout: "centered",
  },
  tags: ["autodocs"],
  argTypes: {},
} satisfies Meta<typeof LogsTable>;

export default meta;
type Story = StoryObj<typeof meta>;

const commonArgs = {
  columns: [
    {
      accessorKey: "timestamp",
      header: "Timestamp",
    },
    {
      accessorKey: "status",
      header: "Status",
    },
    {
      accessorKey: "message",
      header: "message",
    },
  ],
  data: logData,
  isFetching: false,
  selectColumns: true,
  showFiltering: true,
};

export const Table: Story = {
  args: {
    ...commonArgs,
  },
};
