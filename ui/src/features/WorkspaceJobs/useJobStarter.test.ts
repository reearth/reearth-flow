import { renderHook } from "@testing-library/react";
import { describe, expect, test, vi } from "vitest";

import type { Workspace } from "@flow/types";
import { Role } from "@flow/types";

import useJobStarter from "./useJobStarter";

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

const starter = () => renderHook(() => useJobStarter()).result.current;

describe("useJobStarter", () => {
  describe("runBy", () => {
    test("is the workspace member who ran the job", () => {
      expect(starter().runBy({ runById: "u1" })).toEqual(
        expect.objectContaining({ id: "u1", name: "Alice" }),
      );
    });

    test("keeps a user who left the workspace, by placeholder", () => {
      expect(starter().runBy({ runById: "gone" })).toEqual({
        id: "gone",
        name: "Unknown user",
      });
    });

    test("is unset when no user ran the job", () => {
      expect(starter().runBy({})).toBeUndefined();
    });
  });

  describe("triggerName", () => {
    test("is the trigger's description", () => {
      expect(
        starter().triggerName({
          triggerId: "t1",
          triggerDescription: "Nightly import",
        }),
      ).toBe("Nightly import");
    });

    test("says the trigger was deleted when it can no longer be loaded", () => {
      expect(starter().triggerName({ triggerId: "t1" })).toBe(
        "Deleted trigger",
      );
    });

    test("is unset when no trigger started the job", () => {
      expect(starter().triggerName({})).toBeUndefined();
    });
  });
});
