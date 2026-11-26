import { render, screen } from "@testing-library/react";
import { describe, expect, test, vi } from "vitest";

import { EditorProvider } from "@flow/features/Editor/editorContext";
import type { EditorContextType } from "@flow/features/Editor/editorContext";
import type * as PermissionsModule from "@flow/lib/permissions";
import { Role } from "@flow/types";

// vi.mock calls below are hoisted by vitest, so this import still gets the mocks.
import ActionBar from "./index";

// Deploying and toggling sharing are maintainer and owner only. Copying an
// already-issued share URL is a read, and stays available.

const roleState = vi.hoisted(() => ({ role: "writer" as Role }));

vi.mock("@flow/lib/permissions", async (importOriginal) => {
  const actual = await importOriginal<typeof PermissionsModule>();
  return {
    ...actual,
    usePermissions: () => actual.permissionsFor(roleState.role),
  };
});

vi.mock("@flow/features/NotificationSystem/useToast", () => ({
  useToast: () => ({ toast: vi.fn() }),
}));

const renderBar = ({
  role,
  showDialog,
}: {
  role: Role;
  showDialog?: "deploy" | "share";
}) => {
  roleState.role = role;
  return render(
    <EditorProvider
      value={
        {
          isLocked: false,
          isReaderRestricted: role === Role.Reader,
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
        onProjectShareOpen={vi.fn()}
        onProjectExport={vi.fn()}
        onProjectSnapshotSave={vi.fn()}
        onProjectLockChange={vi.fn()}
      />
    </EditorProvider>,
  );
};

// These are icon buttons with no text, so they are addressed by position.
// Render order: 0 deploy, 1 share, 2 additional-actions menu.
const deployButton = () => screen.getAllByRole("button")[0];

describe("ActionBar reader restrictions", () => {
  test.each([Role.Reader, Role.Writer])(
    "a %s cannot open the deploy popover",
    (role) => {
      renderBar({ role });

      expect(deployButton()).toBeDisabled();
    },
  );

  test("a maintainer can open the deploy popover", () => {
    renderBar({ role: Role.Maintainer });

    expect(deployButton()).toBeEnabled();
  });

  test.each([Role.Reader, Role.Writer])(
    "a %s cannot toggle sharing, but can copy an existing link",
    (role) => {
      renderBar({ role, showDialog: "share" });

      expect(screen.getByRole("switch")).toHaveAttribute(
        "aria-disabled",
        "true",
      );
      expect(screen.getByRole("button", { name: /Copy URL/i })).toBeEnabled();
    },
  );

  test("a maintainer can toggle sharing", () => {
    renderBar({ role: Role.Maintainer, showDialog: "share" });

    expect(screen.getByRole("switch")).not.toHaveAttribute(
      "aria-disabled",
      "true",
    );
  });
});
