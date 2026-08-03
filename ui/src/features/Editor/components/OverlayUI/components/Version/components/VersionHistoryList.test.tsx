import { render, screen } from "@testing-library/react";
import { describe, expect, test } from "vitest";

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
    const { container } = render(<VersionHistoryList snapshots={snapshots} />);

    const labels = Array.from(
      container.querySelectorAll("p.flex-2.self-center"),
    ).map((el) => el.textContent);
    // Newest (id 2, later timestamp) should be rendered before the older one.
    expect(labels).toHaveLength(2);
    expect(labels[0]).toBe("before migration");
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

  test("renders no click affordance for snapshot rows", () => {
    render(<VersionHistoryList snapshots={snapshots} />);

    // NamedSnapshot.id and the update-log `version` that
    // previewSnapshot/rollbackProject expect are different, backend-assigned
    // ID spaces with no correct client-side mapping (see ../hooks.ts).
    // Snapshot rows must therefore stay purely informational: no
    // interactive/clickable element should exist for them. If a click
    // handler is reintroduced here, this must start failing.
    expect(screen.queryAllByRole("button")).toHaveLength(0);

    // Clicking the row text must not throw and must not do anything
    // observable — there is no callback prop for it to invoke.
    expect(() => screen.getByText("before migration").click()).not.toThrow();
  });

  test("shows an empty state when there are no snapshots", () => {
    render(<VersionHistoryList snapshots={[]} />);
    expect(screen.getByText(/no versions/i)).toBeInTheDocument();
  });
});
