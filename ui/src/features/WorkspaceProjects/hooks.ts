import { useNavigate } from "@tanstack/react-router";
import { useEffect, useRef, useState } from "react";

import { useProject } from "@flow/lib/gql";
import { useCurrentProject, useCurrentWorkspace } from "@flow/stores";
import { Project } from "@flow/types";
import { OrderDirection } from "@flow/types/paginationOptions";

export default () => {
  const ref = useRef<HTMLDivElement>(null);

  const [workspace] = useCurrentWorkspace();

  const [currentProject, setCurrentProject] = useCurrentProject();
  const [currentOrder, setCurrentOrder] = useState<OrderDirection>(
    OrderDirection.Desc,
  );

  const navigate = useNavigate({ from: "/workspaces/$workspaceId" });
  const { useGetWorkspaceProjects, deleteProject, updateProject } =
    useProject();
  const [currentPage, setCurrentPage] = useState<number>(1);

  const { page, refetch, isFetching } = useGetWorkspaceProjects(workspace?.id, {
    page: currentPage,
    orderDir: currentOrder,
    orderBy: "updatedAt",
  });

  useEffect(() => {
    refetch();
  }, [currentPage, currentOrder, refetch]);

  const totalPages = page?.totalPages as number;
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

  const projects = page?.projects;

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
    currentOrder,
    setCurrentOrder,
    isFetching,
  };
};
