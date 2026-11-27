import { roleOf } from "@flow/lib/permissions";
import { Role } from "@flow/types";
import type { Workspace } from "@flow/types";

const PROJECT_CREATOR_ROLES: Role[] = [Role.Owner, Role.Maintainer];

export const canCreateProjectsIn = (
  workspace: Workspace,
  userId: string | undefined,
): boolean => {
  const role = roleOf(workspace, userId);
  return !!role && PROJECT_CREATOR_ROLES.includes(role);
};

export const getProjectCreatableWorkspaces = (
  workspaces: Workspace[] | undefined,
  userId: string | undefined,
): Workspace[] =>
  (workspaces ?? []).filter((w) => canCreateProjectsIn(w, userId));
