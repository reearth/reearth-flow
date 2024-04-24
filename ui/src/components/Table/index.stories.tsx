import type { Meta, StoryObj } from "@storybook/react";
import { ArrowUpDown } from "lucide-react";

import { Button, Checkbox } from "@flow/components";

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

export const SelectColumns: Story = {
  args: {
    ...commonArgs,
    selectColumns: true,
  },
};

export const SortTable: Story = {
  args: {
    ...commonArgs,
    columns: [
      {
        accessorKey: "status",
        header: "Status",
      },
      {
        // More sorting can be added to the table
        accessorKey: "email",
        header: ({ column }) => {
          return (
            <Button
              variant="ghost"
              onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}>
              Email
              <ArrowUpDown className="ml-2 h-4 w-4" />
            </Button>
          );
        },
      },
      {
        accessorKey: "amount",
        header: "Amount",
      },
    ],
  },
};

export const SelectRows: Story = {
  args: {
    ...commonArgs,
    columns: [
      {
        id: "status",
        header: ({ table }) => (
          <Checkbox
            checked={
              table.getIsAllPageRowsSelected() ||
              (table.getIsSomePageRowsSelected() && "indeterminate")
            }
            onCheckedChange={value => table.toggleAllPageRowsSelected(!!value)}
            aria-label="Select all"
          />
        ),
        cell: ({ row }) => (
          <Checkbox
            checked={row.getIsSelected()}
            onCheckedChange={value => row.toggleSelected(!!value)}
            aria-label="Select row"
          />
        ),
        enableSorting: false,
        enableHiding: false,
      },
      ...commonArgs.columns,
    ],
  },
};

export const ShowPagination: Story = {
  args: {
    ...commonArgs,
    data: [...Array(100).keys()].map(_ => ({
      amount: Math.floor(Math.random() * 300),
      email: Math.random().toString(36).slice(2, 7) + "@mail.com",
      id: Math.random().toString(36).slice(2, 7),
      status: ["success", "failure", "pending", "canceled"][Math.floor(Math.random() * 4)],
    })),
    showPagination: true,
  },
};

export const ShowFiltering: Story = {
  args: {
    ...commonArgs,
    data: [...Array(100).keys()].map(_ => ({
      amount: Math.floor(Math.random() * 300),
      email: Math.random().toString(36).slice(2, 7) + "@mail.com",
      id: Math.random().toString(36).slice(2, 7),
      status: ["success", "failure", "pending", "canceled"][Math.floor(Math.random() * 4)],
    })),
    showFiltering: true,
  },
};
