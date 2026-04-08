import { useParams } from "@tanstack/react-router";
import { isEqual } from "lodash-es";
import { useEffect } from "react";

import { LoadingSplashscreen } from "@flow/components";
import { useProject } from "@flow/lib/gql";
import { useCurrentProject, useCurrentWorkspace } from "@flow/stores";

import NotFoundPage from "../NotFound";

type Props = {
  children: React.ReactNode;
};

const DebugIdWrapper: React.FC<Props> = ({ children }) => {
  const [currentWorkspace] = useCurrentWorkspace();
  const [currentProject, setCurrentProject] = useCurrentProject();

  const { debugId }: { debugId: string } = useParams({
    strict: false,
  });

  const { useGetProject } = useProject();
  const { project, isLoading } = useGetProject(debugId);

  useEffect(() => {
    if (isEqual(currentProject, project)) return;

    setCurrentProject(project);
  }, [project, currentProject, currentWorkspace, setCurrentProject]);

  return isLoading ? (
    <LoadingSplashscreen />
  ) : !project ? (
    <NotFoundPage message={`Debug run with id: "${debugId}" not found.`} />
  ) : currentWorkspace && project.workspaceId !== currentWorkspace.id ? (
    <NotFoundPage
      message={`Debug run : "${project.name}" not found in the workspace "${currentWorkspace?.name}"`}
    />
  ) : (
    children
  );
};

export { DebugIdWrapper };
