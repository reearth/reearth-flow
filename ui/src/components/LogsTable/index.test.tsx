import { fireEvent, render, screen } from "@testing-library/react";
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

  // LogsTable does not set `manualPagination`, so it is the table that would
  // actually be cut short if a `paginatedRowModel` were ever registered in the
  // shared feature set: v9's model slices to `pageSize` (10 by default) without
  // consulting that flag, and nothing about it fails to typecheck.
  test("renders every log, without a page limit", () => {
    const many: UserFacingLog[] = Array.from({ length: 25 }, (_, i) => ({
      ...logs[0],
      nodeId: `node-${i}`,
      message: `message-${i}`,
    }));

    render(
      <LogsTable
        columns={[{ accessorKey: "message", header: "Message" }]}
        data={many}
        isFetching={false}
      />,
    );

    expect(screen.getAllByRole("cell")).toHaveLength(25);
  });

  describe("following new logs", () => {
    const columns = [{ accessorKey: "message", header: "Message" }];
    const makeLogs = (n: number): UserFacingLog[] =>
      Array.from({ length: n }, (_, i) => ({
        ...logs[0],
        nodeId: `node-${i}`,
        message: `message-${i}`,
      }));

    const renderScrollable = (n: number) => {
      const view = render(
        <LogsTable columns={columns} data={makeLogs(n)} isFetching={false} />,
      );
      const scroller = view.container.querySelector(
        ".overflow-auto",
      ) as HTMLDivElement;
      Object.defineProperty(scroller, "clientHeight", { value: 100 });
      Object.defineProperty(scroller, "scrollHeight", {
        get: () => screen.getAllByRole("row").length * 20,
      });
      const rerender = (m: number) =>
        view.rerender(
          <LogsTable columns={columns} data={makeLogs(m)} isFetching={false} />,
        );
      return { scroller, rerender };
    };

    test("stays at the bottom when logs arrive while scrolled to the bottom", () => {
      const { scroller, rerender } = renderScrollable(10);
      scroller.scrollTop = scroller.scrollHeight - scroller.clientHeight;
      fireEvent.scroll(scroller);

      rerender(15);

      expect(scroller.scrollTop).toBe(scroller.scrollHeight);
    });

    test("keeps the user's position once they have scrolled up", () => {
      const { scroller, rerender } = renderScrollable(10);
      scroller.scrollTop = 40;
      fireEvent.scroll(scroller);

      rerender(15);

      expect(scroller.scrollTop).toBe(40);
    });

    test("resumes following after scrolling back to the bottom", () => {
      const { scroller, rerender } = renderScrollable(10);
      scroller.scrollTop = 40;
      fireEvent.scroll(scroller);
      rerender(12);
      scroller.scrollTop = scroller.scrollHeight - scroller.clientHeight;
      fireEvent.scroll(scroller);

      rerender(20);

      expect(scroller.scrollTop).toBe(scroller.scrollHeight);
    });
  });
});
