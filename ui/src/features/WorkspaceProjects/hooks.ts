import { useNavigate } from "@tanstack/react-router";
import { useEffect, useRef, useState } from "react";

import { useProject } from "@flow/lib/gql";
import { useCurrentProject, useCurrentWorkspace } from "@flow/stores";
import { Project } from "@flow/types";

export default () => {
  const ref = useRef<HTMLDivElement>(null);

  const [workspace] = useCurrentWorkspace();

  const [currentProject, setCurrentProject] = useCurrentProject();
  const PROJECTS_FETCH_RATE_PER_PAGE = 5;
  const navigate = useNavigate({ from: "/workspaces/$workspaceId" });
  const { useGetWorkspaceProjects, deleteProject, updateProject } =
    useProject();
  const [currentPage, setCurrentPage] = useState<number>(1);

  const { pages, refetch } = useGetWorkspaceProjects(workspace?.id, {
    pageSize: PROJECTS_FETCH_RATE_PER_PAGE,
    page: currentPage,
  });

  useEffect(() => {
    refetch();
  }, [currentPage, refetch]);

  const totalPages = pages?.totalPages as number;
  const [openProjectAddDialog, setOpenProjectAddDialog] = useState(false);
  const [showError, setShowError] = useState(false);
  const [buttonDisabled, setButtonDisabled] = useState(false);
  const [projectToBeDeleted, setProjectToBeDeleted] = useState<
    string | undefined
  >(undefined);
  const [editProject, setEditProject] = useState<undefined | Project>(
    undefined,
  );

  const handleProjectSelect = (p: Project) => {
    setCurrentProject(p);
    navigate({ to: `/workspaces/${workspace?.id}/projects/${p.id}` });
  };

  const handleDeleteProject = async (id: string) => {
    if (!workspace) return;
    setProjectToBeDeleted(undefined);
    await deleteProject(id, workspace.id);
  };

  const handleUpdateValue = (key: "name" | "description", value: string) => {
    if (!editProject) return;
    setEditProject({ ...editProject, [key]: value });
  };

  const handleUpdateProject = async () => {
    if (!editProject || !editProject.name) return;
    setShowError(false);
    setButtonDisabled(true);

    const { project } = await updateProject({
      projectId: editProject.id,
      name: editProject.name,
      description: editProject.description,
    });

    if (!project) {
      setShowError(true);
      setButtonDisabled(false);
      return;
    }

    setButtonDisabled(false);
    setShowError(false);
    setEditProject(undefined);
    return;
  };

  const projects = pages?.projects;

  return {
    projects,
    ref,
    currentProject,
    projectToBeDeleted,
    editProject,
    showError,
    buttonDisabled,
    openProjectAddDialog,
    setOpenProjectAddDialog,
    setEditProject,
    setProjectToBeDeleted,
    handleProjectSelect,
    handleDeleteProject,
    handleUpdateValue,
    handleUpdateProject,
    currentPage,
    setCurrentPage,
    totalPages,
    PROJECTS_FETCH_RATE_PER_PAGE,
  };
};
