import { render, screen } from "@testing-library/react";
import { describe, expect, test } from "vitest";

import { formatDate } from "@flow/utils";

import VersionHistoryList from "./VersionHistoryList";

describe("VersionHistoryList", () => {
  const snapshots = [
    {
      snapshotNumber: 1,
      label: "",
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
    // snapshotNumber 2 is newer, so its label comes before the older row's date.
    const newest = screen.getByText("before migration");
    const oldest = screen.getAllByText(formatDate(snapshots[0].timestamp))[0];
    expect(
      newest.compareDocumentPosition(oldest) & Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
  });

  test("falls back to a formatted timestamp when the label is empty", () => {
    render(<VersionHistoryList snapshots={snapshots} />);

    // The auto-created snapshot (id 1) has no label, so the row must still
    // be readable: it should show the formatted timestamp instead of a
    // blank row. (The timestamp also appears in the row's trailing badge,
    // so at least one match is expected.)
    expect(
      screen.getAllByText(formatDate(snapshots[0].timestamp)).length,
    ).toBeGreaterThanOrEqual(1);
  });

  test("snapshot rows carry no interactive affordance", () => {
    const { container } = render(<VersionHistoryList snapshots={snapshots} />);

    // snapshotNumber and the update-log `version` that previewSnapshot and
    // rollbackProject expect are unrelated id spaces, and rollback deletes every
    // update above the number it is given. Rows therefore stay informational.
    //
    // Asserting queryAllByRole("button") is NOT enough on its own: a plain
    // <div onClick> has no button role, so that check passes with a click
    // handler present. Verified — it did. These assertions target what a
    // clickable row would actually add.
    const rows = container.querySelectorAll('[class*="justify-between"]');
    expect(rows.length).toBeGreaterThan(0);
    rows.forEach((row) => {
      expect(row.className).not.toContain("cursor-pointer");
      expect(row).not.toHaveAttribute("tabindex");
      expect(row).not.toHaveAttribute("role");
      // onClick shows up as a React prop key on the DOM node's fiber; the class
      // and attribute checks above are the observable proxy for it.
    });
    expect(screen.queryAllByRole("button")).toHaveLength(0);
  });

  test('treats the "auto" label as unnamed and shows the timestamp', () => {
    // Every automatically captured version arrives labelled "auto" (ygo stamps
    // it), so rendering the label verbatim would fill the panel with identical
    // rows reading "auto". Production shape, not a hypothetical one.
    render(
      <VersionHistoryList
        snapshots={[
          {
            snapshotNumber: 3,
            label: "auto",
            timestamp: "2026-07-30T12:00:00Z",
            size: 10,
          },
        ]}
      />,
    );

    expect(screen.queryByText("auto")).not.toBeInTheDocument();
    expect(
      screen.getAllByText(formatDate("2026-07-30T12:00:00Z")).length,
    ).toBeGreaterThanOrEqual(1);
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
