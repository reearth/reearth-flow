import { useParams } from "@tanstack/react-router";
import { useEffect } from "react";

import { Loading } from "@flow/components";
import { useProject } from "@flow/lib/gql";
import { useCurrentProject, useCurrentWorkspace } from "@flow/stores";

import NotFoundPage from "../NotFound";

type Props = {
  children: React.ReactNode;
};

const ProjectIdWrapper: React.FC<Props> = ({ children }) => {
  const [currentWorkspace] = useCurrentWorkspace();
  const [currentProject, setCurrentProject] = useCurrentProject();

  const { projectId }: { projectId: string } = useParams({
    strict: false,
  });

  const { useGetProject } = useProject();
  const { project, isLoading } = useGetProject(projectId);

  useEffect(() => {
    if (
      !project ||
      project.workspaceId !== currentWorkspace?.id ||
      currentProject?.id === project.id
    )
      return;

    setCurrentProject(project);
  }, [project, setCurrentProject, currentProject, currentWorkspace]);

  return isLoading ? (
    <Loading />
  ) : !project ? (
    <NotFoundPage message={`Project with id: "${projectId}" not found.`} />
  ) : currentWorkspace && project.workspaceId !== currentWorkspace.id ? (
    <NotFoundPage
      message={`Project : "${project.name}" not found in the workspace "${currentWorkspace?.name}"`}
    />
  ) : (
    children
  );
};

export { ProjectIdWrapper };
