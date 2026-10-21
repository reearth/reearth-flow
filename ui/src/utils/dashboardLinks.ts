import { config } from "@flow/config";

// Workspace and account management live in the Re:Earth Dashboard wherever
// one is configured (dev and prod), as in Visualizer and CMS. Deployments
// without a Dashboard (OSS, test) keep managing them inside Flow.
//
// Each link returns undefined when there is no Dashboard, so callers fall back
// to the in-app page.

const dashboardLink = (path: string): string | undefined => {
  const base = config().reearthDashboardUrl;
  if (!base) return undefined;
  return `${base.replace(/\/+$/, "")}/${path.replace(/^\/+/, "")}`;
};

export const dashboardNewWorkspaceUrl = () => dashboardLink("new-workspace");

export const dashboardWorkspaceSettingsUrl = (workspaceAlias: string) =>
  dashboardLink(`${encodeURIComponent(workspaceAlias)}/settings/general`);

export const dashboardWorkspaceMembersUrl = (workspaceAlias: string) =>
  dashboardLink(`${encodeURIComponent(workspaceAlias)}/members`);

export const dashboardAccountUrl = () => dashboardLink("settings/profile");
