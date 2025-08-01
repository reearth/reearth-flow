import { useState } from "react";

import { useCms } from "@flow/lib/gql/cms";
import { CMS_ITEMS_FETCH_RATE } from "@flow/lib/gql/cms/useQueries";

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
    CMS_ITEMS_FETCH_RATE,
  );
  const cmsItems = projectId && modelId ? itemsQuery.page?.cmsItems || [] : [];
  const cmsItemsTotalCount = itemsQuery.page?.totalCount || 0;

  const cmsItemsTotalPages = Math.ceil(
    cmsItemsTotalCount / CMS_ITEMS_FETCH_RATE,
  );

  const isLoading =
    projectsQuery.isFetching ||
    (projectId && modelsQuery.isFetching) ||
    (projectId && modelId && itemsQuery.isFetching);

  return {
    cmsProjects,
    cmsModels,
    cmsItems,
    cmsItemsTotalPages,
    currentPage,
    setCurrentPage,
    isLoading,
    viewMode,
    setViewMode,
  };
};
