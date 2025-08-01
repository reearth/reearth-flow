import { useState } from "react";

import { useCms } from "@flow/lib/gql/cms";

type ViewMode = "projects" | "models" | "items";

export default ({
  workspaceId,
  projectId,
  modelId,
}: {
  workspaceId: string;
  projectId?: string;
  modelId?: string;
}) => {
  const { useGetCmsProjects, useGetCmsModels, useGetCmsItems } = useCms();
  const [currentPage, setCurrentPage] = useState(1);
  const [viewMode, setViewMode] = useState<ViewMode>("projects");
  const projectsQuery = useGetCmsProjects(workspaceId, true);
  const cmsProjects = projectsQuery.page?.cmsProjects || [];

  const modelsQuery = useGetCmsModels(projectId || "");
  const cmsModels = projectId ? modelsQuery.page?.cmsModels || [] : [];

  const itemsQuery = useGetCmsItems(
    projectId || "",
    modelId || "",
    currentPage,
    1,
  );
  const cmsItems = projectId && modelId ? itemsQuery.page?.cmsItems || [] : [];
  const cmsItemsTotalCount = itemsQuery.page?.totalCount || 0;

  const isLoading =
    projectsQuery.isFetching ||
    (projectId && modelsQuery.isFetching) ||
    (projectId && modelId && itemsQuery.isFetching);

  return {
    cmsProjects,
    cmsModels,
    cmsItems,
    cmsItemsTotalCount,
    currentPage,
    setCurrentPage,
    isLoading,
    viewMode,
    setViewMode,
  };
};
