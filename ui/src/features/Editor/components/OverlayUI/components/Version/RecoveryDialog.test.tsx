import { render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, test, vi } from "vitest";

import ProjectRecoveryDialog from "./RecoveryDialog";

// What each test sets on the mocked hook; reset before every test.
const hook = vi.hoisted(() => ({
  selectedProjectSnapshotVersion: null as number | null,
  isRefusedVersion: false,
  isEmptyPreview: false,
}));

// recoveryHooks reaches the network and the yjs document; neither is under test
// here. Mirrors what the dialog destructures. previewDocRef matters:
// handleDialogClose dereferences .current, so a missing ref would throw rather
// than fail clearly.
vi.mock("./recoveryHooks", () => ({
  default: () => ({
    history: [],
    latestProjectSnapshotVersion: undefined,
    previewDocRef: { current: null },
    previewDocYWorkflows: null,
    selectedProjectSnapshotVersion: hook.selectedProjectSnapshotVersion,
    isFetching: false,
    isLoadingPreview: false,
    isReverting: false,
    isCorruptedVersion: false,
    isRefusedVersion: hook.isRefusedVersion,
    isEmptyPreview: hook.isEmptyPreview,
    openVersionConfirmationDialog: false,
    setOpenVersionConfirmationDialog: vi.fn(),
    onProjectRollback: vi.fn(),
    onVersionSelection: vi.fn(),
    onWorkflowCorruption: vi.fn(),
  }),
}));

// Revert is disabled for readers too, so without this every "Revert is
// disabled" assertion below would pass for the wrong reason.
vi.mock("@flow/lib/permissions", () => ({
  usePermissions: () => ({ revertVersion: true }),
}));

vi.mock("./components/VersionEditorComponent", () => ({
  default: () => <div data-testid="preview-canvas" />,
}));

const revertButton = () => screen.getByRole("button", { name: "Revert" });

describe("ProjectRecoveryDialog", () => {
  beforeEach(() => {
    hook.selectedProjectSnapshotVersion = null;
    hook.isRefusedVersion = false;
    hook.isEmptyPreview = false;
  });

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

  // The control for the two tests below: an ordinary preview can be reverted.
  test("lets a previewed version be reverted", () => {
    hook.selectedProjectSnapshotVersion = 2;
    render(<ProjectRecoveryDialog yDoc={null} onDialogClose={vi.fn()} />);

    expect(screen.getByTestId("preview-canvas")).toBeInTheDocument();
    expect(revertButton()).toBeEnabled();
  });

  // A refused entry used to preview as an empty canvas with Revert enabled, and
  // reverting deleted every workflow in the project.
  test("shows a refused version as a message, with Revert disabled", () => {
    hook.selectedProjectSnapshotVersion = 2;
    hook.isRefusedVersion = true;
    render(<ProjectRecoveryDialog yDoc={null} onDialogClose={vi.fn()} />);

    expect(screen.queryByTestId("preview-canvas")).not.toBeInTheDocument();
    expect(
      screen.getByText("This version can no longer be restored"),
    ).toBeInTheDocument();
    expect(revertButton()).toBeDisabled();
  });

  test("disables Revert for a preview with no workflows", () => {
    hook.selectedProjectSnapshotVersion = 2;
    hook.isEmptyPreview = true;
    render(<ProjectRecoveryDialog yDoc={null} onDialogClose={vi.fn()} />);

    expect(
      screen.getByText("This version has no workflows"),
    ).toBeInTheDocument();
    expect(revertButton()).toBeDisabled();
  });
});
