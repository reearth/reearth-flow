import { render, screen } from "@testing-library/react";
import { describe, expect, test, vi } from "vitest";

import { formatDate } from "@flow/utils";

import VersionHistoryList from "./VersionHistoryList";

describe("VersionHistoryList", () => {
  const snapshots = [
    {
      snapshotNumber: 1,
      label: "auto",
      timestamp: "2026-07-30T09:00:00Z",
      size: 100,
    },
    {
      snapshotNumber: 2,
      label: "before migration",
      timestamp: "2026-07-30T10:00:00Z",
      size: 120,
    },
  ];

  test("renders every snapshot, newest first", () => {
    render(<VersionHistoryList snapshots={snapshots} />);

    // Assert on visible text order rather than CSS selectors: restyling must not
    // break this, and a real ordering regression must not hide behind markup.
    const newest = screen.getByText("before migration");
    const oldest = screen.getByText("Snapshot 1");
    expect(
      newest.compareDocumentPosition(oldest) & Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
  });

  // The predecessor of this test asserted getAllByText(date).length >= 1, which
  // was true whether the date rendered once or twice. It therefore accommodated a
  // bug where the trailing badge repeated the row's own date and snapshotNumber
  // was never displayed at all. Exact counts, so that cannot recur.
  test("each row shows its snapshot number once and its date once", () => {
    render(<VersionHistoryList snapshots={snapshots} />);

    expect(screen.getAllByText("Snapshot 1")).toHaveLength(1);
    expect(screen.getAllByText("Snapshot 2")).toHaveLength(1);

    for (const snapshot of snapshots) {
      expect(screen.getAllByText(formatDate(snapshot.timestamp))).toHaveLength(
        1,
      );
    }
  });

  test('renders the "auto" label as Autosaved, never verbatim', () => {
    // Every automatically captured version arrives labelled "auto" (ygo stamps
    // it), so rendering the label verbatim would fill the panel with identical
    // rows reading "auto". Production shape, not a hypothetical one.
    render(<VersionHistoryList snapshots={snapshots} />);

    expect(screen.queryByText("auto")).not.toBeInTheDocument();
    expect(screen.getByText("Autosaved")).toBeInTheDocument();
    // The user-named row keeps its own label.
    expect(screen.getByText("before migration")).toBeInTheDocument();
  });

  test("selecting a row reports that row's snapshotNumber", () => {
    // The number passed here is fed to projectNamedSnapshot. Passing a row's
    // index, or the update-log version, would read an unrelated point in history.
    const onSnapshotSelect = vi.fn();
    render(
      <VersionHistoryList
        snapshots={snapshots}
        onSnapshotSelect={onSnapshotSelect}
      />,
    );

    screen.getByText("before migration").click();
    expect(onSnapshotSelect).toHaveBeenCalledTimes(1);
    expect(onSnapshotSelect).toHaveBeenCalledWith(2);
  });

  test("rows are reachable and operable by keyboard", () => {
    const onSnapshotSelect = vi.fn();
    const { container } = render(
      <VersionHistoryList
        snapshots={snapshots}
        onSnapshotSelect={onSnapshotSelect}
      />,
    );

    const rows = container.querySelectorAll('[role="button"]');
    expect(rows.length).toBe(2);
    rows.forEach((row) => expect(row).toHaveAttribute("tabindex", "0"));
  });

  test("marks the selected row as pressed", () => {
    const { container } = render(
      <VersionHistoryList snapshots={snapshots} selectedSnapshotNumber={2} />,
    );

    const pressed = container.querySelectorAll('[aria-pressed="true"]');
    expect(pressed).toHaveLength(1);
    expect(pressed[0].textContent).toContain("before migration");
  });

  test("shows an empty state when there are no snapshots", () => {
    render(<VersionHistoryList snapshots={[]} />);
    expect(screen.getByText(/no versions/i)).toBeInTheDocument();
  });

  test("distinguishes a failed load from an empty history", () => {
    // "No versions yet" on a failed query tells the user their history does not
    // exist, when in fact it could not be loaded.
    render(<VersionHistoryList snapshots={[]} isError />);
    expect(screen.queryByText(/no versions yet/i)).not.toBeInTheDocument();
    expect(screen.getByText(/could not load/i)).toBeInTheDocument();
  });
});
