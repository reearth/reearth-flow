import { useNavigate } from "@tanstack/react-router";
import { useMemo, useRef, useState } from "react";

import { useProject } from "@flow/lib/gql";
import { useCurrentProject, useCurrentWorkspace } from "@flow/stores";
import { Project } from "@flow/types";

import usePagination from "../hooks/usePagination";

const PROJECT_FETCH_RATE = 5;
export default () => {
  const ref = useRef<HTMLDivElement>(null);

  const [workspace] = useCurrentWorkspace();

  const [currentProject, setCurrentProject] = useCurrentProject();
  const [currentPage, setCurrentPage] = useState<number>(0);
  const navigate = useNavigate({ from: "/workspaces/$workspaceId" });
  const { useGetWorkspaceProjectsInfinite, deleteProject, updateProject } =
    useProject();

  const { pages, hasNextPage, fetchNextPage, isFetchingNextPage } =
    useGetWorkspaceProjectsInfinite(workspace?.id, PROJECT_FETCH_RATE);

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

  const projects: Project[] | undefined = useMemo(
    () => pages?.[currentPage]?.projects,
    [pages, currentPage],
  );

  const { totalPages, handleNextPage, handlePrevPage, canGoNext } =
    usePagination<Project>(
      PROJECT_FETCH_RATE,
      hasNextPage,
      isFetchingNextPage,
      pages,
      fetchNextPage,
      currentPage,
      setCurrentPage,
    );

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
    totalPages,
    currentPage,
    hasNextPage: canGoNext,
    // isFetching,
    isFetchingNextPage,
    handleNextPage,
    handlePrevPage,
  };
};
