import { afterEach, beforeEach, describe, expect, it } from "vitest";

import { isRestrictedWorkspace } from "./useRestrictedWorkspaces";

describe("isRestrictedWorkspace", () => {
  afterEach(() => {
    window.REEARTH_CONFIG = undefined;
  });

  describe("with a dashboard configured", () => {
    beforeEach(() => {
      window.REEARTH_CONFIG = {
        reearthDashboardUrl: "https://dashboard.example.com",
      };
    });

    it("is a team workspace whose policy does not allow private projects", () => {
      expect(isRestrictedWorkspace({ personal: false }, false)).toBe(true);
    });

    it("is not a team workspace whose policy allows private projects", () => {
      expect(isRestrictedWorkspace({ personal: false }, true)).toBe(false);
    });

    it("is never a personal workspace, whatever its policy", () => {
      expect(isRestrictedWorkspace({ personal: true }, false)).toBe(false);
    });

    it("treats a policy still loading or that failed as available", () => {
      expect(isRestrictedWorkspace({ personal: false }, undefined)).toBe(false);
    });
  });

  it("leaves every workspace available without a dashboard", () => {
    window.REEARTH_CONFIG = {};

    expect(isRestrictedWorkspace({ personal: false }, false)).toBe(false);
  });
});
