import type { Meta, StoryObj } from "@storybook/react-vite";

import { UserFacingLog, UserFacingLogLevel } from "@flow/types";

import { LogsTable } from ".";

export const logData: UserFacingLog[] = [
  {
    jobId: "1",
    level: UserFacingLogLevel.Info,
    message: "Job started",
    timestamp: "2021-09-01T00:00:00Z",
    nodeId: "node-1",
    nodeName: "Node 1",
  },
  {
    jobId: "1",
    level: UserFacingLogLevel.Info,
    message: "Job started",
    timestamp: "2021-09-01T00:00:00Z",
    nodeId: "node-1",
    nodeName: "Node 1",
  },
  {
    jobId: "1",
    level: UserFacingLogLevel.Info,
    message: "Job started",
    timestamp: "2021-09-01T00:00:00Z",
    nodeId: "node-1",
    nodeName: "Node 1",
  },
  {
    jobId: "1",
    level: UserFacingLogLevel.Info,
    message: "Job started",
    timestamp: "2021-09-01T00:00:00Z",
    nodeId: "node-1",
    nodeName: "Node 1",
  },
  {
    jobId: "1",
    level: UserFacingLogLevel.Info,
    message: "Job started",
    timestamp: "2021-09-01T00:00:00Z",
    nodeId: "node-1",
    nodeName: "Node 1",
  },
  {
    jobId: "1",
    level: UserFacingLogLevel.Info,
    message: "Job started",
    timestamp: "2021-09-01T00:00:00Z",
    nodeId: "node-1",
    nodeName: "Node 1",
  },
  {
    jobId: "1",
    level: UserFacingLogLevel.Info,
    message: "Job started",
    timestamp: "2021-09-01T00:00:00Z",
    nodeId: "node-1",
    nodeName: "Node 1",
  },
];

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
      accessorKey: "level",
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
