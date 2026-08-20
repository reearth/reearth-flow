import { useParams } from "@tanstack/react-router";
import { useEffect } from "react";

import { LoadingSplashscreen } from "@flow/components";
import { useUser, useWorkspace } from "@flow/lib/gql";
import { useCurrentUserRole, useCurrentWorkspace } from "@flow/stores";
import { UserMember } from "@flow/types";

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

  const { useGetWorkspace } = useWorkspace();
  const { workspace, isLoading } = useGetWorkspace(workspaceId);
  const { useGetMe } = useUser();
  const { me } = useGetMe();

  const userMembers = workspace?.members as UserMember[];
  const userRole = userMembers?.find(
    (m) => "userId" in m && m.userId === me?.id,
  )?.role;

  useEffect(() => {
    if (!workspace) return;
    setCurrentWorkspace(workspace);
    setCurrentUserRole(userRole);
  }, [workspace, userRole, setCurrentUserRole, setCurrentWorkspace]);

  return !isLoading && !workspace ? (
    <NotFoundPage message={`Workspace with id: "${workspaceId}" not found.`} />
  ) : isLoading || !currentWorkspace ? (
    <LoadingSplashscreen />
  ) : (
    children
  );
};

export { WorkspaceIdWrapper };
