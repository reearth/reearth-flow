import { afterEach, describe, expect, it } from "vitest";

import {
  dashboardAccountUrl,
  dashboardNewWorkspaceUrl,
  dashboardWorkspaceMembersUrl,
  dashboardWorkspaceSettingsUrl,
} from "./dashboardLinks";

const setDashboardUrl = (reearthDashboardUrl?: string) => {
  window.REEARTH_CONFIG = { reearthDashboardUrl };
};

describe("dashboard links", () => {
  afterEach(() => {
    window.REEARTH_CONFIG = undefined;
  });

  it("links to the Dashboard's management pages", () => {
    setDashboardUrl("https://dashboard.example.com");

    expect(dashboardNewWorkspaceUrl()).toBe(
      "https://dashboard.example.com/new-workspace",
    );
    expect(dashboardWorkspaceSettingsUrl("my-team")).toBe(
      "https://dashboard.example.com/my-team/settings/general",
    );
    expect(dashboardWorkspaceMembersUrl("my-team")).toBe(
      "https://dashboard.example.com/my-team/members",
    );
    expect(dashboardAccountUrl()).toBe(
      "https://dashboard.example.com/settings/profile",
    );
  });

  it("joins a base URL with a trailing slash", () => {
    setDashboardUrl("https://dashboard.example.com/");

    expect(dashboardNewWorkspaceUrl()).toBe(
      "https://dashboard.example.com/new-workspace",
    );
  });

  it("returns undefined without a Dashboard, so Flow keeps its own pages", () => {
    setDashboardUrl(undefined);

    expect(dashboardNewWorkspaceUrl()).toBeUndefined();
    expect(dashboardWorkspaceSettingsUrl("my-team")).toBeUndefined();
    expect(dashboardWorkspaceMembersUrl("my-team")).toBeUndefined();
    expect(dashboardAccountUrl()).toBeUndefined();
  });
});
