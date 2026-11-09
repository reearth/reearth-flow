import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, test, vi } from "vitest";

import type { Workspace } from "@flow/types";
import { Role } from "@flow/types";

import { CreatorFilter } from "./CreatorFilter";

const workspace = {
  id: "w1",
  name: "w",
  members: [
    {
      userId: "u1",
      role: Role.Owner,
      user: { id: "u1", name: "Alice", email: "a@x" },
    },
    {
      userId: "u2",
      role: Role.Writer,
      user: { id: "u2", name: "Bob", email: "b@x" },
    },
  ],
} as Workspace;

vi.mock("@flow/stores", async (importOriginal) => ({
  ...(await importOriginal<Record<string, unknown>>()),
  useCurrentWorkspace: () => [workspace, vi.fn()],
}));

describe("CreatorFilter", () => {
  test("adds a picked member to the creators", async () => {
    const onChange = vi.fn();
    render(<CreatorFilter value={["u2"]} onChange={onChange} />);

    const user = userEvent.setup();
    await user.type(screen.getByRole("combobox"), "ali");
    await user.click(await screen.findByRole("option", { name: /Alice/ }));

    expect(onChange).toHaveBeenCalledWith(["u2", "u1"]);
  });

  test("removes a creator by its chip", () => {
    const onChange = vi.fn();
    render(<CreatorFilter value={["u1", "u2"]} onChange={onChange} />);

    fireEvent.click(screen.getByRole("button", { name: "Remove Bob" }));

    expect(onChange).toHaveBeenCalledWith(["u1"]);
  });

  test("shows a creator who has left the workspace as unknown", () => {
    render(<CreatorFilter value={["gone"]} onChange={vi.fn()} />);
    expect(screen.getByText("Unknown user")).toBeInTheDocument();
  });
});
