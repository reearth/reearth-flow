import { useMemo } from "react";

import { config } from "@flow/config";
import { useWorkspace } from "@flow/lib/gql";
import type { Workspace } from "@flow/types";

// A restricted workspace is a team (non-personal) workspace whose policy does
// not allow private projects. Flow does not let users work in these: they are
// disabled in the workspace pickers and their routes redirect to the personal
// workspace.
//
// The restriction applies only when an external account dashboard is
// configured (reearthDashboardUrl), since that is where a workspace's policy
// is managed. Without one every workspace is available and no policy is asked.
//
// A workspace whose policy is still loading or failed to load is treated as
// available, like the server's policy checks: a slow or unreachable policy
// service must not lock users out.
export const isWorkspaceRestrictionEnabled = () =>
  !!config().reearthDashboardUrl;

export const isRestrictedWorkspace = (
  workspace: Pick<Workspace, "personal">,
  canCreatePrivateProject: boolean | undefined,
) =>
  isWorkspaceRestrictionEnabled() &&
  !workspace.personal &&
  canCreatePrivateProject === false;

export default (workspaces?: Workspace[]) => {
  const { useGetWorkspacePolicies } = useWorkspace();

  const teamWorkspaceIds = useMemo(
    () =>
      isWorkspaceRestrictionEnabled()
        ? (workspaces?.filter((w) => !w.personal).map((w) => w.id) ?? [])
        : [],
    [workspaces],
  );
  const { policies, isLoading } = useGetWorkspacePolicies(teamWorkspaceIds);

  const restrictedWorkspaceIds = new Set(
    (workspaces ?? [])
      .filter((w) =>
        isRestrictedWorkspace(w, policies[w.id]?.canCreatePrivateProject),
      )
      .map((w) => w.id),
  );

  return { restrictedWorkspaceIds, isLoading };
};
