import { useT } from "@flow/lib/i18n";
import { useCurrentUserRole } from "@flow/stores";
import { Role } from "@flow/types";
import type { UserMember, Workspace } from "@flow/types";

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

export const roleOf = (
  workspace: Workspace | undefined,
  userId: string | undefined,
): Role | undefined => {
  if (!workspace || !userId) return undefined;
  return workspace.members?.find(
    (m): m is UserMember => "userId" in m && m.userId === userId,
  )?.role;
};

export const useRoleLabel = () => {
  const t = useT();
  return (role: Role | undefined): string | undefined => {
    switch (role) {
      case Role.Owner:
        return t("Owner");
      case Role.Maintainer:
        return t("Maintainer");
      case Role.Writer:
        return t("Writer");
      case Role.Reader:
        return t("Reader");
      default:
        return undefined;
    }
  };
};
