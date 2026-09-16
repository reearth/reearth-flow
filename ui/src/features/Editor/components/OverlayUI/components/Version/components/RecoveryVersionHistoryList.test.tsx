import { render, screen } from "@testing-library/react";
import { describe, expect, test, vi } from "vitest";

import RecoveryVersionHistoryList from "./RecoveryVersionHistoryList";

describe("RecoveryVersionHistoryList", () => {
  const latestProjectSnapshotVersion = {
    id: "doc-1",
    timestamp: "2026-07-30T11:00:00Z",
    updates: [],
    version: 3,
  };

  const history = [
    { timestamp: "2026-07-30T10:00:00Z", version: 2 },
    { timestamp: "2026-07-30T09:00:00Z", version: 1 },
    // The live head is also present in the raw update-log history feed;
    // it must be filtered out of the clickable list below (pre-existing
    // behaviour, restored unchanged).
    { timestamp: "2026-07-30T11:00:00Z", version: 3 },
  ];

  test("filters the live head out of the clickable list", () => {
    render(
      <RecoveryVersionHistoryList
        latestProjectSnapshotVersion={latestProjectSnapshotVersion}
        history={history}
        selectedProjectSnapshotVersion={null}
        onVersionSelection={vi.fn()}
      />,
    );

    // Only versions 1 and 2 should appear as rows; version 3 is the head
    // and is shown once, in the "Current Version" block, not the list.
    expect(screen.getAllByText(/Version\s*3/)).toHaveLength(1);
    expect(screen.getByText(/Version\s*2/)).toBeInTheDocument();
    expect(screen.getByText(/Version\s*1/)).toBeInTheDocument();
  });

  test("clicking a row invokes onVersionSelection with the real update-log version", () => {
    const onVersionSelection = vi.fn();
    render(
      <RecoveryVersionHistoryList
        latestProjectSnapshotVersion={latestProjectSnapshotVersion}
        history={history}
        selectedProjectSnapshotVersion={null}
        onVersionSelection={onVersionSelection}
      />,
    );

    // Row order follows the raw history array (no client-side re-sort
    // here, unlike the snapshot list), so version 2's row is first.
    screen.getByText(/Version\s*2/).click();

    expect(onVersionSelection).toHaveBeenCalledWith(2);
  });

  test("renders nothing below the head when there is no other history", () => {
    render(
      <RecoveryVersionHistoryList
        latestProjectSnapshotVersion={latestProjectSnapshotVersion}
        history={[{ timestamp: "2026-07-30T11:00:00Z", version: 3 }]}
        selectedProjectSnapshotVersion={null}
        onVersionSelection={vi.fn()}
      />,
    );

    // Recovery mode restores the pre-existing behaviour verbatim: no
    // "empty state" message, just nothing rendered below the head.
    expect(screen.queryByText(/no versions/i)).not.toBeInTheDocument();
  });
});
