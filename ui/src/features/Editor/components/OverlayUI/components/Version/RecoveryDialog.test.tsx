import { render } from "@testing-library/react";
import { describe, expect, test, vi } from "vitest";

import ProjectRecoveryDialog from "./RecoveryDialog";

// recoveryHooks reaches the network and the yjs document; neither is under test
// here. What is under test is that the dialog can MOUNT.
vi.mock("./recoveryHooks", () => ({
  default: () => ({
    history: [],
    isFetching: false,
    latestProjectSnapshotVersion: undefined,
    selectedProjectSnapshotVersion: undefined,
    previewedProjectSnapshotVersion: undefined,
    isLoadingPreview: false,
    isCorruptedVersion: false,
    openVersionConfirmationDialog: false,
    setOpenVersionConfirmationDialog: vi.fn(),
    handleProjectSnapshotSelect: vi.fn(),
    handleProjectRollback: vi.fn(),
  }),
}));

describe("ProjectRecoveryDialog", () => {
  // This dialog renders from the route's errorComponent, which REPLACES the
  // whole route subtree — so in production it is mounted outside any
  // EditorProvider. It previously called useEditorContext, which throws when no
  // provider is present, so opening it from the corruption screen threw during
  // render and the only revert path in the product was unreachable.
  //
  // Rendering with no provider is therefore the configuration that matters, and
  // nothing covered it before. Keep this test free of context wrappers.
  test("mounts with no EditorProvider ancestor", () => {
    expect(() =>
      render(<ProjectRecoveryDialog yDoc={null} onDialogClose={vi.fn()} />),
    ).not.toThrow();
  });
});
