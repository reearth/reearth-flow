import { useNavigate } from "@tanstack/react-router";
import { useRef, useState } from "react";

import { useProjectDuplicate, useProjectPagination } from "@flow/hooks";
import { useProject } from "@flow/lib/gql";
import { useCurrentWorkspace } from "@flow/stores";
import type { Project, Workspace } from "@flow/types";

export default () => {
  const ref = useRef<HTMLDivElement>(null);

  const [workspace] = useCurrentWorkspace();
  const [duplicateProject, setDuplicateProject] = useState<undefined | Project>(
    undefined,
  );

  const { isDuplicating, handleProjectDuplication: duplicate } =
    useProjectDuplicate(duplicateProject);

  const navigate = useNavigate({ from: "/workspaces/$workspaceId" });

  const handleProjectDuplication = async (
    project: Project,
    targetWorkspace: Workspace,
  ) => {
    const success = await duplicate(project, targetWorkspace);
    if (success && targetWorkspace.id !== workspace?.id) {
      navigate({ to: `/workspaces/${targetWorkspace.id}/projects` });
    }
  };
  const { deleteProject, setProjectStar } = useProject();

  const {
    currentPage,
    projects,
    totalPages,
    isFetching,
    currentOrderBy,
    currentSortValue,
    searchTerm,
    orderDirections,
    sortOptions,
    isDebouncingSearch,
    setCurrentPage,
    setSearchTerm,
    handleSortChange,
  } = useProjectPagination({ workspace });

  const [openProjectAddDialog, setOpenProjectAddDialog] = useState(false);
  const [openProjectDuplicateDialog, setOpenProjectDuplicateDialog] =
    useState(false);
  const [projectToBeDeleted, setProjectToBeDeleted] = useState<
    string | undefined
  >(undefined);
  const [editProject, setEditProject] = useState<undefined | Project>(
    undefined,
  );

  const handleProjectSelect = (p: Project) =>
    navigate({ to: `/workspaces/${workspace?.id}/projects/${p.id}` });

  const handleDeleteProject = async (id: string) => {
    if (!workspace) return;
    setProjectToBeDeleted(undefined);
    await deleteProject(id, workspace.id);
  };

  const handleProjectStar = (p: Project, starred: boolean) =>
    setProjectStar(p.id, starred);

  return {
    projects,
    ref,
    projectToBeDeleted,
    editProject,
    duplicateProject,
    openProjectAddDialog,
    openProjectDuplicateDialog,
    currentPage,
    totalPages,
    isFetching,
    isDuplicating,
    currentOrderBy,
    currentSortValue,
    orderDirections,
    searchTerm,
    sortOptions,
    isDebouncingSearch,
    setSearchTerm,
    setOpenProjectAddDialog,
    setOpenProjectDuplicateDialog,
    setEditProject,
    setDuplicateProject,
    setProjectToBeDeleted,
    setCurrentPage,
    handleProjectDuplication,
    handleProjectSelect,
    handleDeleteProject,
    handleProjectStar,
    handleSortChange,
  };
};
