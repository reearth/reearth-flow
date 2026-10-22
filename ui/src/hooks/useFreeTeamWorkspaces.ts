import { useMemo } from "react";

import { config } from "@flow/config";
import { useWorkspace } from "@flow/lib/gql";
import type { Workspace } from "@flow/types";

// PROVISIONAL (free-tier policy, not yet confirmed): this restriction and how
// a free team is detected are subject to change once the ecosystem's plans for
// Flow are decided. Search for "PROVISIONAL (free-tier policy" to find every
// place it applies.
//
// A free team workspace is a team (non-personal) workspace whose plan does not
// allow private projects, which is how the dashboard's free plan reads. Flow
// does not let users work in these: they are disabled in the workspace pickers
// and their routes redirect to the personal workspace.
//
// The restriction applies only where a Dashboard is configured (dev and prod),
// since that is where free teams can upgrade. Without one (test, OSS) every
// workspace is available and no policy is asked.
//
// A workspace whose policy is still loading or failed to load is treated as
// available, like the server's plan checks: a slow or unreachable policy
// service must not lock paying teams out.
export const isFreeTeamRestrictionEnabled = () =>
  !!config().reearthDashboardUrl;

export const isFreeTeamWorkspace = (
  workspace: Pick<Workspace, "personal">,
  canCreatePrivateProject: boolean | undefined,
) =>
  isFreeTeamRestrictionEnabled() &&
  !workspace.personal &&
  canCreatePrivateProject === false;

export default (workspaces?: Workspace[]) => {
  const { useGetWorkspacePolicies } = useWorkspace();

  const teamWorkspaceIds = useMemo(
    () =>
      isFreeTeamRestrictionEnabled()
        ? (workspaces?.filter((w) => !w.personal).map((w) => w.id) ?? [])
        : [],
    [workspaces],
  );
  const { policies, isLoading } = useGetWorkspacePolicies(teamWorkspaceIds);

  const freeTeamWorkspaceIds = new Set(
    (workspaces ?? [])
      .filter((w) =>
        isFreeTeamWorkspace(w, policies[w.id]?.canCreatePrivateProject),
      )
      .map((w) => w.id),
  );

  return { freeTeamWorkspaceIds, isLoading };
};
