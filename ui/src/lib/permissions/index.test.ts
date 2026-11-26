import { describe, expect, it } from "vitest";

import { Role } from "@flow/types";

import { permissionsFor } from ".";
import type { Permissions } from ".";

const allowed = (role: Role | undefined) =>
  (Object.entries(permissionsFor(role)) as [keyof Permissions, boolean][])
    .filter(([, v]) => v)
    .map(([k]) => k)
    .sort();

const writerUp: (keyof Permissions)[] = [
  "cancelJob",
  "editWorkflow",
  "manageAssets",
  "revertVersion",
  "runDebug",
  "viewMembers",
];
const maintainerUp: (keyof Permissions)[] = [
  ...writerUp,
  "lockProject",
  "manageDeployments",
  "manageMembers",
  "manageProjects",
  "manageTriggers",
  "shareProject",
];

describe("permissionsFor", () => {
  it("denies everything when the role is unknown", () => {
    expect(allowed(undefined)).toEqual([]);
  });

  it("gives readers no write permissions", () => {
    expect(allowed(Role.Reader)).toEqual([]);
  });

  it("lets writers build workflows but not deploy or manage", () => {
    expect(allowed(Role.Writer)).toEqual([...writerUp].sort());
  });

  it("lets maintainers manage everything but the workspace itself", () => {
    expect(allowed(Role.Maintainer)).toEqual([...maintainerUp].sort());
  });

  it("lets owners do everything", () => {
    expect(allowed(Role.Owner)).toEqual(
      [...maintainerUp, "manageWorkspace"].sort(),
    );
  });
});
