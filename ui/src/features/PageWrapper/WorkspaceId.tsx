import { useParams } from "@tanstack/react-router";
import { useEffect } from "react";

import { Loading } from "@flow/components";
import { useWorkspace } from "@flow/lib/gql";
import { useCurrentWorkspace } from "@flow/stores";

import NotFoundPage from "../NotFoundPage";

type Props = {
  children: React.ReactNode;
};

const WorkspaceIdWrapper: React.FC<Props> = ({ children }) => {
  const [currentWorkspace, setCurrentWorkspace] = useCurrentWorkspace();

  const { workspaceId }: { workspaceId: string } = useParams({
    strict: false,
  });

  const { useGetWorkspace } = useWorkspace();
  const { workspace, isLoading } = useGetWorkspace(workspaceId);

  useEffect(() => {
    if (!workspace) return;
    if (currentWorkspace?.id === workspace.id) return;
    setCurrentWorkspace(workspace);

    return;
  }, [workspace, currentWorkspace, setCurrentWorkspace]);

  if (isLoading) return <Loading />;

  if (!workspace)
    return <NotFoundPage message={`Workspace with id: "${workspaceId}" not found.`} />;

  return children;
};

export { WorkspaceIdWrapper };
