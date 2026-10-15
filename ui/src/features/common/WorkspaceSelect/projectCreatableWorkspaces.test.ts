import { Role } from "@flow/types";
import type { Member, Workspace } from "@flow/types";

import {
  canCreateProjectsIn,
  getProjectCreatableWorkspaces,
} from "./projectCreatableWorkspaces";

const workspace = (id: string, members: Member[]): Workspace => ({
  id,
  name: id,
  personal: false,
  members,
});

const ME = "user-me";

describe("canCreateProjectsIn", () => {
  test.each([
    [Role.Owner, true],
    [Role.Maintainer, true],
    [Role.Writer, false],
    [Role.Reader, false],
  ])("%s → %s", (role, expected) => {
    const ws = workspace("w", [{ userId: ME, role }]);
    expect(canCreateProjectsIn(ws, ME)).toBe(expected);
  });

  test("uses the user's own membership, not someone else's", () => {
    const ws = workspace("w", [
      { userId: "someone-else", role: Role.Owner },
      { userId: ME, role: Role.Reader },
    ]);
    expect(canCreateProjectsIn(ws, ME)).toBe(false);
  });

  test("is false when the user is not a member or unknown", () => {
    const ws = workspace("w", [{ userId: "someone-else", role: Role.Owner }]);
    expect(canCreateProjectsIn(ws, ME)).toBe(false);
    expect(canCreateProjectsIn(ws, undefined)).toBe(false);
  });
});

describe("getProjectCreatableWorkspaces", () => {
  test("keeps only workspaces the user may create projects in, in order", () => {
    const workspaces = [
      workspace("owned", [{ userId: ME, role: Role.Owner }]),
      workspace("read", [{ userId: ME, role: Role.Reader }]),
      workspace("maintained", [{ userId: ME, role: Role.Maintainer }]),
      workspace("written", [{ userId: ME, role: Role.Writer }]),
    ];
    expect(
      getProjectCreatableWorkspaces(workspaces, ME).map((w) => w.id),
    ).toEqual(["owned", "maintained"]);
  });

  test("handles workspaces still loading", () => {
    expect(getProjectCreatableWorkspaces(undefined, ME)).toEqual([]);
  });
});
