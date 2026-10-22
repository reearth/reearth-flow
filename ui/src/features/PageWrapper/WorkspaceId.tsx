import { useNavigate, useParams } from "@tanstack/react-router";
import { useEffect } from "react";

import { LoadingSplashscreen } from "@flow/components";
import {
  isFreeTeamRestrictionEnabled,
  isFreeTeamWorkspace,
} from "@flow/hooks/useFreeTeamWorkspaces";
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

  // PROVISIONAL (free-tier policy, not yet confirmed): free team workspaces are
  // not available in Flow, so a link or bookmark to one goes to the personal
  // workspace instead. See useFreeTeamWorkspaces.
  const { policy, isLoading: isPolicyLoading } = useGetWorkspacePolicy(
    isFreeTeamRestrictionEnabled() && workspace && !workspace.personal
      ? workspace.id
      : undefined,
  );
  const isFreeTeam =
    !!workspace &&
    isFreeTeamWorkspace(workspace, policy?.canCreatePrivateProject);

  const userMembers = workspace?.members as UserMember[];
  const userRole = userMembers?.find(
    (m) => "userId" in m && m.userId === me?.id,
  )?.role;

  useEffect(() => {
    if (!isFreeTeam || !me?.myWorkspaceId) return;
    navigate({ to: `/workspaces/${me.myWorkspaceId}/projects`, replace: true });
  }, [isFreeTeam, me?.myWorkspaceId, navigate]);

  useEffect(() => {
    if (!workspace || isFreeTeam) return;
    setCurrentWorkspace(workspace);
    setCurrentUserRole(userRole);
  }, [
    workspace,
    isFreeTeam,
    userRole,
    setCurrentUserRole,
    setCurrentWorkspace,
  ]);

  return !isLoading && !workspace ? (
    <NotFoundPage message={`Workspace with id: "${workspaceId}" not found.`} />
  ) : isLoading || isPolicyLoading || isFreeTeam || !currentWorkspace ? (
    <LoadingSplashscreen />
  ) : (
    children
  );
};

export { WorkspaceIdWrapper };
