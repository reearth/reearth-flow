import { useParams } from "@tanstack/react-router";
import { useEffect } from "react";

import { Loading } from "@flow/components";
import { useProject } from "@flow/lib/gql";
import { useCurrentProject, useCurrentWorkspace } from "@flow/stores";

import NotFoundPage from "../NotFoundPage";

type Props = {
  children: React.ReactNode;
};

const ProjectIdWrapper: React.FC<Props> = ({ children }) => {
  const [currentWorkspace] = useCurrentWorkspace();
  const [, setCurrentProject] = useCurrentProject();

  const { projectId }: { projectId: string } = useParams({
    strict: false,
  });

  const { useGetProject } = useProject();
  const { project, isLoading } = useGetProject(projectId);

  useEffect(() => {
    if (!project) return;

    if (project.workspaceId != currentWorkspace?.id) return;

    setCurrentProject(project);
    return;
  }, [project, setCurrentProject, currentWorkspace]);

  if (isLoading || !project) return <Loading />;

  if (!project) return <NotFoundPage message={`Project with id: "${projectId}" not found.`} />;

  if (project.workspaceId != currentWorkspace?.id)
    return (
      <NotFoundPage
        message={`Project : "${project.name}" not found in the workspace "${currentWorkspace?.name}"`}
      />
    );

  return <>{children}</>;
};

export { ProjectIdWrapper };
