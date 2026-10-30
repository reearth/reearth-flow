import { renderHook } from "@testing-library/react";
import { describe, expect, test, vi } from "vitest";

import type { Project, Workspace } from "@flow/types";
import { Role } from "@flow/types";

import useProjectAuthors from "./useProjectAuthors";

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

const project = (createdById?: string, updatedById?: string) =>
  ({ id: "p1", createdById, updatedById }) as Project;

describe("useProjectAuthors", () => {
  test("resolves both authors from the workspace members", () => {
    const { result } = renderHook(() => useProjectAuthors(project("u1", "u2")));
    expect(result.current.createdBy).toEqual(
      expect.objectContaining({ name: "Alice" }),
    );
    expect(result.current.updatedBy).toEqual(
      expect.objectContaining({ name: "Bob" }),
    );
  });

  test("falls back to the creator before anyone else edits", () => {
    const { result } = renderHook(() => useProjectAuthors(project("u1")));
    expect(result.current.updatedBy?.name).toBe("Alice");
  });

  test("keeps an author who left the workspace, by placeholder", () => {
    const { result } = renderHook(() =>
      useProjectAuthors(project("gone", "gone")),
    );
    expect(result.current.createdBy).toEqual({
      id: "gone",
      name: "Unknown user",
    });
  });

  test("has no authors on a project from before they were recorded", () => {
    const { result } = renderHook(() => useProjectAuthors(project()));
    expect(result.current.createdBy).toBeUndefined();
    expect(result.current.updatedBy).toBeUndefined();
  });
});
