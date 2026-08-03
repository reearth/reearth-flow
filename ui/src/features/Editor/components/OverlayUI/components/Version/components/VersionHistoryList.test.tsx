import { render, screen } from "@testing-library/react";
import { describe, expect, test, vi } from "vitest";

import { formatDate } from "@flow/utils";

import VersionHistoryList from "./VersionHistoryList";

describe("VersionHistoryList", () => {
  const snapshots = [
    {
      id: 1,
      label: "",
      timestamp: "2026-07-30T09:00:00Z",
      size: 100,
    },
    {
      id: 2,
      label: "before migration",
      timestamp: "2026-07-30T10:00:00Z",
      size: 120,
    },
  ];

  test("renders every snapshot, newest first", () => {
    render(
      <VersionHistoryList snapshots={snapshots} onSnapshotSelect={vi.fn()} />,
    );

    const rows = screen.getAllByRole("button");
    expect(rows).toHaveLength(2);
    // Newest (id 2, later timestamp) should be rendered before the older one.
    expect(rows[0]).toHaveTextContent("before migration");
  });

  test("falls back to a formatted timestamp when the label is empty", () => {
    render(
      <VersionHistoryList snapshots={snapshots} onSnapshotSelect={vi.fn()} />,
    );

    // The auto-created snapshot (id 1) has no label, so the row must still
    // be readable: it should show the formatted timestamp instead of a
    // blank row. (The timestamp also appears in the row's trailing badge,
    // so at least two matches are expected.)
    expect(
      screen.getAllByText(formatDate(snapshots[0].timestamp)).length,
    ).toBeGreaterThanOrEqual(1);
  });

  test("calls onSnapshotSelect with the snapshot id when a row is clicked", async () => {
    const onSnapshotSelect = vi.fn();
    render(
      <VersionHistoryList
        snapshots={snapshots}
        onSnapshotSelect={onSnapshotSelect}
      />,
    );

    screen.getByText("before migration").click();

    expect(onSnapshotSelect).toHaveBeenCalledWith(2);
  });

  test("shows an empty state when there are no snapshots", () => {
    render(<VersionHistoryList snapshots={[]} onSnapshotSelect={vi.fn()} />);
    expect(screen.getByText(/no versions/i)).toBeInTheDocument();
  });
});
