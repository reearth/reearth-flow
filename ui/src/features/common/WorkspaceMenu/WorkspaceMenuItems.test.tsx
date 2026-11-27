import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, test, vi } from "vitest";

import { DropdownMenu, DropdownMenuContent } from "@flow/components";
import { Role } from "@flow/types";
import type { Workspace } from "@flow/types";

import { WorkspaceMenuItems } from "./WorkspaceMenuItems";

const me = { id: "u1" };

const workspace = (
  id: string,
  name: string,
  role: Role,
  personal = false,
): Workspace =>
  ({
    id,
    name,
    personal,
    members: [{ userId: me.id, role }],
  }) as unknown as Workspace;

const workspaces = [
  workspace("w-personal", "John Doe", Role.Owner, true),
  workspace("w-eukarya", "Eukarya", Role.Reader),
  workspace("w-analytics", "Analytics Project", Role.Writer),
  workspace("w-design", "Design Studio", Role.Maintainer),
];

vi.mock("@flow/lib/gql", () => ({
  useUser: () => ({ useGetMe: () => ({ me }) }),
  useWorkspace: () => ({ useGetWorkspaces: () => ({ workspaces }) }),
}));

vi.mock("@flow/hooks", () => ({
  useRestrictedWorkspaces: () => ({ restrictedWorkspaceIds: new Set() }),
}));

vi.mock("@flow/stores", async (importOriginal) => ({
  ...(await importOriginal<Record<string, unknown>>()),
  useCurrentWorkspace: () => [workspaces[1], vi.fn()],
  useCurrentUserRole: () => [Role.Reader, vi.fn()],
}));

vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => vi.fn(),
}));

const renderMenu = () =>
  render(
    <DropdownMenu open>
      <DropdownMenuContent>
        <WorkspaceMenuItems onNewWorkspace={vi.fn()} />
      </DropdownMenuContent>
    </DropdownMenu>,
  );

describe("WorkspaceMenuItems", () => {
  test("shows the user's role under the current workspace", () => {
    renderMenu();

    const current = screen.getByText("Eukarya").parentElement as HTMLElement;
    expect(within(current).getByText("Reader")).toBeInTheDocument();
  });

  test("shows the user's role under every workspace in the switcher", async () => {
    renderMenu();

    fireEvent.click(screen.getByText("Eukarya"));

    for (const [name, label] of [
      ["John Doe", "Owner"],
      ["Analytics Project", "Writer"],
      ["Design Studio", "Maintainer"],
    ]) {
      const row = (await screen.findByText(name)).parentElement as HTMLElement;
      expect(within(row).getByText(label)).toBeInTheDocument();
    }
  });
});
