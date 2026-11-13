import { renderHook } from "@testing-library/react";
import { describe, expect, test, vi } from "vitest";

import type { Workspace } from "@flow/types";
import { Role } from "@flow/types";

import useJobRunBy from "./useJobRunBy";

const workspace = {
  id: "w1",
  name: "w",
  members: [
    {
      userId: "u1",
      role: Role.Owner,
      user: { id: "u1", name: "Alice", email: "a@x" },
    },
  ],
} as Workspace;

vi.mock("@flow/stores", async (importOriginal) => ({
  ...(await importOriginal<Record<string, unknown>>()),
  useCurrentWorkspace: () => [workspace, vi.fn()],
}));

const runBy = (job: Parameters<ReturnType<typeof useJobRunBy>>[0]): string => {
  const { result } = renderHook(() => useJobRunBy());
  return result.current(job);
};

describe("useJobRunBy", () => {
  test("names the user who ran the job", () => {
    expect(runBy({ runById: "u1" })).toBe("Alice");
  });

  test("names the user over the trigger when both are set", () => {
    expect(
      runBy({
        runById: "u1",
        triggerId: "t1",
        triggerDescription: "From CMS",
      }),
    ).toBe("Alice");
  });

  test("keeps a user who left the workspace, by placeholder", () => {
    expect(runBy({ runById: "gone" })).toBe("Unknown user");
  });

  test("names the trigger when no user ran the job", () => {
    expect(
      runBy({ triggerId: "t1", triggerDescription: "Nightly import" }),
    ).toBe("Trigger: Nightly import");
  });

  test("says the trigger was deleted when it can no longer be loaded", () => {
    expect(runBy({ triggerId: "t1" })).toBe("Deleted trigger");
  });

  test("has nothing to name for a job from before it was recorded", () => {
    expect(runBy({})).toBe("N/A");
  });
});
