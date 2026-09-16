import { render, screen } from "@testing-library/react";
import { describe, expect, test } from "vitest";

import type { UserFacingLog } from "@flow/types";
import { UserFacingLogLevel } from "@flow/types";

import { LogsTable } from ".";

const logs: UserFacingLog[] = [
  {
    jobId: "job-1",
    timestamp: "2026-09-09T15:21:54Z",
    nodeId: "f33687a6",
    nodeName: "Attribute Manager",
    level: UserFacingLogLevel.Error,
    message: "Attribute Manager - Failed",
  },
];

describe("LogsTable", () => {
  // The panel switches between this table and the diagnostics one in place, so
  // a missing header row here shifted every log line up by a row's height on
  // each switch. Header parity is the fix, hence a test on the headers rather
  // than on the rows.
  test("renders a header row for its columns", () => {
    render(
      <LogsTable
        columns={[
          { accessorKey: "timestamp", header: "Timestamp" },
          { accessorKey: "nodeName", header: "Action Name" },
          { accessorKey: "message", header: "Message" },
        ]}
        data={logs}
        isFetching={false}
      />,
    );

    expect(
      screen.getByRole("columnheader", { name: "Timestamp" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("columnheader", { name: "Action Name" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("columnheader", { name: "Message" }),
    ).toBeInTheDocument();
  });
});
