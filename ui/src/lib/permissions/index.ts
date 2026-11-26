import { useCurrentUserRole } from "@flow/stores";
import { Role } from "@flow/types";

export type Permissions = {
  editWorkflow: boolean;
  runDebug: boolean;
  revertVersion: boolean;
  manageAssets: boolean;
  cancelJob: boolean;
  viewMembers: boolean;
  lockProject: boolean;
  manageProjects: boolean;
  shareProject: boolean;
  manageDeployments: boolean;
  manageTriggers: boolean;
  manageMembers: boolean;
  manageWorkspace: boolean;
};

const RANK: Record<Role, number> = {
  [Role.Reader]: 0,
  [Role.Writer]: 1,
  [Role.Maintainer]: 2,
  [Role.Owner]: 3,
};

export const permissionsFor = (role: Role | undefined): Permissions => {
  const rank = role ? RANK[role] : -1;
  const writer = rank >= RANK[Role.Writer];
  const maintainer = rank >= RANK[Role.Maintainer];
  const owner = rank >= RANK[Role.Owner];

  return {
    editWorkflow: writer,
    runDebug: writer,
    revertVersion: writer,
    manageAssets: writer,
    cancelJob: writer,
    viewMembers: writer,
    lockProject: maintainer,
    manageProjects: maintainer,
    shareProject: maintainer,
    manageDeployments: maintainer,
    manageTriggers: maintainer,
    manageMembers: maintainer,
    manageWorkspace: owner,
  };
};

export const usePermissions = (): Permissions => {
  const [role] = useCurrentUserRole();
  return permissionsFor(role);
};
