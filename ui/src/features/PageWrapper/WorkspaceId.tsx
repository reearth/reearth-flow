import { useParams } from "@tanstack/react-router";
import { useEffect } from "react";

import { LoadingSplashscreen } from "@flow/components";
import { useWorkspace } from "@flow/lib/gql";
import { useCurrentWorkspace } from "@flow/stores";

import NotFoundPage from "../NotFound";

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
    setCurrentWorkspace(workspace);
  }, [workspace, setCurrentWorkspace]);

  return !isLoading && !workspace ? (
    <NotFoundPage message={`Workspace with id: "${workspaceId}" not found.`} />
  ) : isLoading || !currentWorkspace ? (
    <LoadingSplashscreen />
  ) : (
    children
  );
};

export { WorkspaceIdWrapper };
