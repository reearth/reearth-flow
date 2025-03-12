import { useNavigate } from "@tanstack/react-router";
import { useRef, useState } from "react";

import { useProjectPagination } from "@flow/hooks";
import { useProject } from "@flow/lib/gql";
import { useCurrentProject, useCurrentWorkspace } from "@flow/stores";
import { Project } from "@flow/types";

export default () => {
  const ref = useRef<HTMLDivElement>(null);

  const [workspace] = useCurrentWorkspace();

  const [currentProject, setCurrentProject] = useCurrentProject();
  const [isDuplicatingState, setIsDuplicatingState] = useState<boolean>(false);

  const navigate = useNavigate({ from: "/workspaces/$workspaceId" });
  const { deleteProject, updateProject } = useProject();

  const {
    currentPage,
    projects,
    totalPages,
    isFetching,
    currentOrder,
    orderDirections,
    setCurrentPage,
    handleOrderChange,
  } = useProjectPagination({ workspace });

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

  return {
    projects,
    ref,
    currentProject,
    projectToBeDeleted,
    editProject,
    showError,
    buttonDisabled,
    openProjectAddDialog,
    currentPage,
    totalPages,
    isFetching,
    isDuplicatingState,
    currentOrder,
    orderDirections,
    setOpenProjectAddDialog,
    setEditProject,
    setProjectToBeDeleted,
    setCurrentPage,
    setIsDuplicatingState,
    handleProjectSelect,
    handleDeleteProject,
    handleUpdateValue,
    handleUpdateProject,
    handleOrderChange,
  };
};
