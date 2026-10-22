import { afterEach, beforeEach, describe, expect, it } from "vitest";

import { isFreeTeamWorkspace } from "./useFreeTeamWorkspaces";

describe("isFreeTeamWorkspace", () => {
  afterEach(() => {
    window.REEARTH_CONFIG = undefined;
  });

  describe("with a Dashboard (dev, prod)", () => {
    beforeEach(() => {
      window.REEARTH_CONFIG = {
        reearthDashboardUrl: "https://dashboard.example.com",
      };
    });

    it("is a team workspace whose plan does not allow private projects", () => {
      expect(isFreeTeamWorkspace({ personal: false }, false)).toBe(true);
    });

    it("is not a team workspace on a paid plan", () => {
      expect(isFreeTeamWorkspace({ personal: false }, true)).toBe(false);
    });

    it("is never a personal workspace, though its plan is free", () => {
      expect(isFreeTeamWorkspace({ personal: true }, false)).toBe(false);
    });

    it("treats a policy still loading or that failed as available", () => {
      expect(isFreeTeamWorkspace({ personal: false }, undefined)).toBe(false);
    });
  });

  it("leaves every workspace available without a Dashboard (test, OSS)", () => {
    window.REEARTH_CONFIG = {};

    expect(isFreeTeamWorkspace({ personal: false }, false)).toBe(false);
  });
});
