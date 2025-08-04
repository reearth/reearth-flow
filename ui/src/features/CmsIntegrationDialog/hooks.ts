import { useState } from "react";

import { useCms } from "@flow/lib/gql/cms";
import { CMS_ITEMS_FETCH_RATE } from "@flow/lib/gql/cms/useQueries";
import { CmsItem, CmsModel, CmsProject } from "@flow/types";

export type ViewMode =
  | "projects"
  | "models"
  | "items"
  | "itemDetails"
  | "itemAssets";

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

  const navigateTo = (
    mode: ViewMode,
    options?: {
      project?: CmsProject | null;
      model?: CmsModel | null;
      item?: CmsItem | null;
    },
  ) => {
    const { project, model, item } = options || {};

    switch (mode) {
      case "projects":
        setSelectedProject(null);
        setSelectedModel(null);
        setSelectedItem(null);
        break;
      case "models":
        setSelectedProject(project || selectedProject);
        setSelectedModel(null);
        setSelectedItem(null);
        break;
      case "items":
        setSelectedProject(project || selectedProject);
        setSelectedModel(model || selectedModel);
        setSelectedItem(null);
        break;
      case "itemDetails":
      case "itemAssets":
        setSelectedProject(project || selectedProject);
        setSelectedModel(model || selectedModel);
        setSelectedItem(item || null);
        break;
    }

    setViewMode(mode);
    setSearchTerm("");
  };

  const handleProjectSelect = (project: CmsProject) => {
    if (!project?.id) return;
    navigateTo("models", { project });
  };

  const handleModelSelect = (model: CmsModel) => {
    if (!model?.id) return;
    navigateTo("items", { model });
  };

  const handleItemView = (item: CmsItem) => {
    navigateTo("itemDetails", { item });
  };

  const handleAssetView = (item: CmsItem) => {
    navigateTo("itemAssets", { item });
  };

  const handleBackToProjects = () => navigateTo("projects");
  const handleBackToModels = () => navigateTo("models");
  const handleBackToItems = () => navigateTo("items");

  const handleItemDetailClose = () => {
    setIsItemDetailOpen(false);
    navigateTo("items");
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
    handleAssetView,
    handleItemDetailClose,
    handleBackToItems,
  };
};
