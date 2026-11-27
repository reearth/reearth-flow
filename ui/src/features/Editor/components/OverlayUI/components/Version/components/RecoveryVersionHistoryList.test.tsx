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

  test("says there is nothing to restore when only the head is left", () => {
    render(
      <RecoveryVersionHistoryList
        latestProjectSnapshotVersion={latestProjectSnapshotVersion}
        history={[{ timestamp: "2026-07-30T11:00:00Z", version: 3 }]}
        selectedProjectSnapshotVersion={null}
        onVersionSelection={vi.fn()}
      />,
    );

    // The server lists only entries it can rebuild, so an empty list is a real
    // answer rather than a gap, and it points to where saved versions live.
    expect(
      screen.getByText(
        "There is nothing to restore here. Saved versions are in the Version History panel.",
      ),
    ).toBeInTheDocument();
  });

  // A failed request is not an empty history: saved entries may exist.
  test("says the history could not be loaded when the request failed", () => {
    render(
      <RecoveryVersionHistoryList
        latestProjectSnapshotVersion={latestProjectSnapshotVersion}
        history={undefined}
        isError
        selectedProjectSnapshotVersion={null}
        onVersionSelection={vi.fn()}
      />,
    );

    expect(
      screen.getByText(
        "The version history could not be loaded. Close this dialog and try again.",
      ),
    ).toBeInTheDocument();
    expect(screen.queryByText(/nothing to restore/)).not.toBeInTheDocument();
  });
});
