import { useState } from "react";

import { useCms } from "@flow/lib/gql/cms";
import { CMS_ITEMS_FETCH_RATE } from "@flow/lib/gql/cms/useQueries";
import { CmsItem, CmsModel, CmsProject } from "@flow/types";

type ViewMode = "projects" | "models" | "items" | "itemDetails" | "itemsAssets";

export default ({ workspaceId }: { workspaceId: string }) => {
  const { useGetCmsProjects, useGetCmsModels, useGetCmsItems } = useCms();
  const [selectedProject, setSelectedProject] = useState<CmsProject | null>(
    null,
  );
  const [selectedModel, setSelectedModel] = useState<CmsModel | null>(null);
  const [selectedItem, setSelectedItem] = useState<CmsItem | null>(null);

  const [currentPage, setCurrentPage] = useState(1);
  const [searchTerm, setSearchTerm] = useState<string>("");
  const [viewMode, setViewMode] = useState<ViewMode>("projects");
  const [isItemDetailOpen, setIsItemDetailOpen] = useState(false);

  const projectsQuery = useGetCmsProjects(workspaceId, true);
  const cmsProjects = projectsQuery.page?.cmsProjects || [];

  const modelsQuery = useGetCmsModels(selectedProject?.id || "");
  const cmsModels = selectedProject?.id
    ? modelsQuery.page?.cmsModels || []
    : [];

  const itemsQuery = useGetCmsItems(
    selectedProject?.id || "",
    selectedModel?.id || "",
    currentPage,
    CMS_ITEMS_FETCH_RATE,
  );
  const cmsItems =
    selectedProject?.id && selectedModel?.id
      ? itemsQuery.page?.cmsItems || []
      : [];
  const cmsItemsTotalCount = itemsQuery.page?.totalCount || 0;

  const cmsItemsTotalPages = Math.ceil(
    cmsItemsTotalCount / CMS_ITEMS_FETCH_RATE,
  );

  const filteredProjects = cmsProjects?.filter(
    (p) =>
      ("id" in p &&
        (p.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
          p.alias.toLowerCase().includes(searchTerm.toLowerCase()))) ||
      p.id.toLowerCase().includes(searchTerm.toLowerCase()),
  ) as CmsProject[];

  const filteredModels = cmsModels?.filter(
    (m) =>
      ("id" in m && m.name.toLowerCase().includes(searchTerm.toLowerCase())) ||
      m.id.toLowerCase().includes(searchTerm.toLowerCase()),
  ) as CmsModel[];

  const isLoading =
    projectsQuery.isFetching ||
    (selectedProject?.id && modelsQuery.isFetching) ||
    (selectedProject?.id && selectedModel?.id && itemsQuery.isFetching);

  const handleProjectSelect = (project: CmsProject) => {
    if (!project?.id) return;
    setSelectedProject(project);
    setSelectedModel(null);
    setViewMode("models");
  };

  const handleModelSelect = (model: CmsModel) => {
    if (!model?.id) return;
    setSelectedModel(model);
    setViewMode("items");
  };

  const handleBackToProjects = () => {
    setSelectedProject(null);
    setSelectedModel(null);
    setSelectedItem(null);
    setViewMode("projects");
  };

  const handleBackToModels = () => {
    setSelectedModel(null);
    setSelectedItem(null);
    setViewMode("models");
  };

  const handleItemView = (item: CmsItem) => {
    setSelectedItem(item);

    setViewMode("itemDetails");
  };

  const handleItemDetailClose = () => {
    setIsItemDetailOpen(false);
    setSelectedItem(null);
    setViewMode("items");
  };

  return {
    selectedProject,
    selectedModel,
    selectedItem,
    filteredProjects,
    filteredModels,
    cmsItems,
    cmsItemsTotalPages,
    currentPage,
    searchTerm,
    isLoading,
    viewMode,
    isItemDetailOpen,
    setSearchTerm,
    setCurrentPage,
    handleProjectSelect,
    handleModelSelect,
    handleBackToProjects,
    handleBackToModels,
    handleItemView,
    handleItemDetailClose,
  };
};
