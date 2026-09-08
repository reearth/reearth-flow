import { render, screen } from "@testing-library/react";
import { describe, expect, test, vi } from "vitest";

import {
  EditorProvider,
  type EditorContextType,
} from "@flow/features/Editor/editorContext";

// vi.mock calls below are hoisted by vitest, so this import still gets the mocks.
import ActionBar from "./index";

// Deploying and toggling sharing both write, so a reader must not reach either.
// Copying an already-issued share URL is a read, and stays available.

vi.mock("@flow/features/NotificationSystem/useToast", () => ({
  useToast: () => ({ toast: vi.fn() }),
}));

const renderBar = ({
  isReaderRestricted,
  showDialog,
}: {
  isReaderRestricted: boolean;
  showDialog?: "deploy" | "share";
}) =>
  render(
    <EditorProvider
      value={
        {
          isLocked: false,
          isReaderRestricted,
          canViewIntermediateData: false,
        } as EditorContextType
      }>
      <ActionBar
        allowedToDeploy
        isSaving={false}
        showDialog={showDialog}
        sharingUrl="https://example.test/shared/abc"
        onDialogOpen={vi.fn()}
        onDialogClose={vi.fn()}
        onWorkflowDeployment={vi.fn()}
        onProjectShare={vi.fn()}
        onProjectExport={vi.fn()}
        onProjectSnapshotSave={vi.fn()}
        onProjectLockChange={vi.fn()}
      />
    </EditorProvider>,
  );

// These are icon buttons with no text, so they are addressed by position.
// Render order: 0 deploy, 1 share, 2 additional-actions menu.
const deployButton = () => screen.getAllByRole("button")[0];

describe("ActionBar reader restrictions", () => {
  test("a reader cannot open the deploy popover", () => {
    renderBar({ isReaderRestricted: true });

    expect(deployButton()).toBeDisabled();
  });

  test("a writer can open the deploy popover", () => {
    renderBar({ isReaderRestricted: false });

    // Same props, only the role differs — so the role is what disables deploy.
    expect(deployButton()).toBeEnabled();
  });

  test("a reader cannot toggle sharing, but can copy an existing link", () => {
    renderBar({ isReaderRestricted: true, showDialog: "share" });

    expect(screen.getByRole("switch")).toHaveAttribute("aria-disabled", "true");
    expect(screen.getByRole("button", { name: /Copy URL/i })).toBeEnabled();
  });

  test("a writer can toggle sharing", () => {
    renderBar({ isReaderRestricted: false, showDialog: "share" });

    expect(screen.getByRole("switch")).not.toHaveAttribute(
      "aria-disabled",
      "true",
    );
  });
});
