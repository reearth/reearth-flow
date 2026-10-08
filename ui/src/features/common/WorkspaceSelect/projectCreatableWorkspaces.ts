import { Role } from "@flow/types";
import type { UserMember, Workspace } from "@flow/types";

// Mirrors the server's RBAC for project creation (rbac/definitions.go):
const PROJECT_CREATOR_ROLES: Role[] = [Role.Owner, Role.Maintainer];

export const canCreateProjectsIn = (
  workspace: Workspace,
  userId: string | undefined,
): boolean => {
  if (!userId) return false;
  const role = workspace.members.find(
    (m): m is UserMember => "userId" in m && m.userId === userId,
  )?.role;
  return !!role && PROJECT_CREATOR_ROLES.includes(role);
};

export const getProjectCreatableWorkspaces = (
  workspaces: Workspace[] | undefined,
  userId: string | undefined,
): Workspace[] =>
  (workspaces ?? []).filter((w) => canCreateProjectsIn(w, userId));
