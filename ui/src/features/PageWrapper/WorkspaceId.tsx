import { useNavigate, useParams } from "@tanstack/react-router";
import { useEffect } from "react";

import { LoadingSplashscreen } from "@flow/components";
import {
  isWorkspaceRestrictionEnabled,
  isRestrictedWorkspace,
} from "@flow/hooks/useRestrictedWorkspaces";
import { useUser, useWorkspace } from "@flow/lib/gql";
import { useCurrentUserRole, useCurrentWorkspace } from "@flow/stores";
import type { UserMember } from "@flow/types";

import NotFoundPage from "../NotFound";

type Props = {
  children: React.ReactNode;
};

const WorkspaceIdWrapper: React.FC<Props> = ({ children }) => {
  const [currentWorkspace, setCurrentWorkspace] = useCurrentWorkspace();
  const [_currentUserRole, setCurrentUserRole] = useCurrentUserRole();

  const { workspaceId }: { workspaceId: string } = useParams({
    strict: false,
  });

  const navigate = useNavigate();
  const { useGetWorkspace, useGetWorkspacePolicy } = useWorkspace();
  const { workspace, isLoading } = useGetWorkspace(workspaceId);
  const { useGetMe } = useUser();
  const { me } = useGetMe();

  // A link or bookmark to a restricted workspace goes to the personal workspace
  // instead. See useRestrictedWorkspaces.
  const { policy, isLoading: isPolicyLoading } = useGetWorkspacePolicy(
    isWorkspaceRestrictionEnabled() && workspace && !workspace.personal
      ? workspace.id
      : undefined,
  );
  const isRestricted =
    !!workspace &&
    isRestrictedWorkspace(workspace, policy?.canCreatePrivateProject);

  const userMembers = workspace?.members as UserMember[];
  const userRole = userMembers?.find(
    (m) => "userId" in m && m.userId === me?.id,
  )?.role;

  useEffect(() => {
    if (!isRestricted || !me?.myWorkspaceId) return;
    navigate({ to: `/workspaces/${me.myWorkspaceId}/projects`, replace: true });
  }, [isRestricted, me?.myWorkspaceId, navigate]);

  useEffect(() => {
    if (!workspace || isRestricted) return;
    setCurrentWorkspace(workspace);
    setCurrentUserRole(userRole);
  }, [
    workspace,
    isRestricted,
    userRole,
    setCurrentUserRole,
    setCurrentWorkspace,
  ]);

  return !isLoading && !workspace ? (
    <NotFoundPage message={`Workspace with id: "${workspaceId}" not found.`} />
  ) : isLoading || isPolicyLoading || isRestricted || !currentWorkspace ? (
    <LoadingSplashscreen />
  ) : (
    children
  );
};

export { WorkspaceIdWrapper };
