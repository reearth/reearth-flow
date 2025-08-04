import { CaretLeftIcon, EyeIcon, StackIcon } from "@phosphor-icons/react";
import { ColumnDef } from "@tanstack/react-table";

import {
  Dialog,
  DialogContent,
  DialogContentSection,
  DialogContentWrapper,
  DialogTitle,
  ScrollArea,
  LoadingSkeleton,
  DataTable as Table,
  FlowLogo,
  IconButton,
  Pagination,
  Input,
  Button,
} from "@flow/components";
import BasicBoiler from "@flow/components/BasicBoiler";
import { useT } from "@flow/lib/i18n";
import { useCurrentWorkspace } from "@flow/stores";
import type { CmsItem } from "@flow/types/cmsIntegration";

import CmsAssetSelector from "./CmsAssetSelector";
import CmsBreadcrumb from "./CmsBreadcrumb";
import CmsItemDetails from "./CmsItemDetails";
import CmsModelCard from "./CmsModelCard";
import CmsProjectCard from "./CmsProjectCard";
import useHooks from "./hooks";

type Props = {
  onDialogClose: () => void;
  onCmsItemValue?: (value: string) => void;
};

const CmsIntegrationDialog: React.FC<Props> = ({
  onDialogClose,
  onCmsItemValue,
}) => {
  const t = useT();
  const [currentWorkspace] = useCurrentWorkspace();

  const {
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
    setSearchTerm,
    setCurrentPage,
    handleProjectSelect,
    handleModelSelect,
    handleBackToProjects,
    handleBackToModels,
    handleItemView,
    handleAssetView,
    handleBackToItems,
  } = useHooks({
    workspaceId: currentWorkspace?.id ?? "",
  });

  const getItemAssets = (item: CmsItem) => {
    if (!selectedModel) return [];

    const assets: { key: string; value: string; field: any }[] = [];
    Object.entries(item.fields).forEach(([key, value]) => {
      const fieldSchema = selectedModel.schema.fields.find(
        (f) => f.key === key,
      );
      if (fieldSchema?.type === "asset" && value) {
        assets.push({ key, value, field: fieldSchema });
      }
    });
    return assets;
  };

  const columns: ColumnDef<CmsItem>[] = selectedModel
    ? [
        {
          accessorKey: "id",
          header: "ID",
        },
        ...selectedModel.schema.fields.map((field) => ({
          accessorKey: `fields.${field.key}`,
          header: field.name,
          cell: ({ row }: any) => {
            const value = row.original.fields[field.key];
            const displayValue =
              typeof value === "object"
                ? JSON.stringify(value).substring(0, 50) + "..."
                : String(value || "-");
            return (
              <div className="max-w-[150px] truncate" title={displayValue}>
                {displayValue}
              </div>
            );
          },
        })),
        {
          accessorKey: "createdAt",
          header: t("Created At"),
        },
        {
          accessorKey: "updatedAt",
          header: t("Updated At"),
        },
        {
          accessorKey: "quickActions",
          header: t("Quick Actions"),
          cell: ({ row }) => {
            const assets = getItemAssets(row.original);

            return (
              <div className="flex gap-1">
                <IconButton
                  icon={<EyeIcon />}
                  onClick={() => handleItemView(row.original)}
                  title={t("View Details")}
                />
                {assets.length > 0 && (
                  <IconButton
                    icon={<StackIcon />}
                    onClick={() => handleAssetView(row.original)}
                    title={`${t("Quick Select Assets")} (${assets.length})`}
                  />
                )}
              </div>
            );
          },
        },
      ]
    : [];

  return (
    <Dialog open onOpenChange={onDialogClose}>
      <DialogContent className="max-h-[90vh] w-full max-w-6xl overflow-hidden">
        <DialogTitle>
          <CmsBreadcrumb
            viewMode={viewMode}
            selectedProject={selectedProject}
            selectedModel={selectedModel}
            selectedItem={selectedItem}
            onBackToProjects={handleBackToProjects}
            onBackToModels={handleBackToModels}
            onBackToItems={handleBackToItems}
          />
        </DialogTitle>
        <DialogContentWrapper>
          <DialogContentSection className="flex h-[600px] flex-col overflow-hidden">
            {viewMode !== "itemDetails" && viewMode !== "itemsAssets" && (
              <div className="mb-3 flex items-center gap-4">
                {viewMode !== "projects" && (
                  <Button
                    className="p-2.5"
                    onClick={
                      viewMode === "models"
                        ? handleBackToProjects
                        : handleBackToModels
                    }
                    variant="outline">
                    <CaretLeftIcon />
                  </Button>
                )}
                {viewMode !== "items" && (
                  <Input
                    placeholder={t("Search") + "..."}
                    value={searchTerm ?? ""}
                    onChange={(e) => setSearchTerm(e.target.value)}
                    className="h-[36px] max-w-sm"
                  />
                )}
              </div>
            )}

            <ScrollArea className="flex-1">
              {viewMode === "projects" && (
                <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
                  {isLoading ? (
                    <LoadingSkeleton />
                  ) : filteredProjects.length === 0 ? (
                    <div className="col-span-full py-8 text-center text-muted-foreground">
                      <BasicBoiler
                        text={t("No Projects Found")}
                        className="[&>div>p]:text-md size-4 pt-60"
                        icon={<FlowLogo className="size-14 text-accent" />}
                      />
                    </div>
                  ) : (
                    filteredProjects.map((project) => {
                      return (
                        <CmsProjectCard
                          key={project.id}
                          project={project}
                          onProjectSelect={handleProjectSelect}
                        />
                      );
                    })
                  )}
                </div>
              )}
              {viewMode === "models" && selectedProject && (
                <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
                  {isLoading ? (
                    <LoadingSkeleton />
                  ) : filteredModels.length === 0 ? (
                    <div className="col-span-full py-8 text-center text-muted-foreground">
                      <BasicBoiler
                        text={t("No Models Found")}
                        className="[&>div>p]:text-md size-4 pt-60"
                        icon={<FlowLogo className="size-14 text-accent" />}
                      />
                    </div>
                  ) : (
                    filteredModels.map((model) => {
                      return (
                        <CmsModelCard
                          key={model.id}
                          model={model}
                          onModelSelect={handleModelSelect}
                        />
                      );
                    })
                  )}
                </div>
              )}
            </ScrollArea>
            {viewMode === "items" && selectedModel && (
              <div className="flex flex-col overflow-hidden">
                <div className="overflow-hidden">
                  {isLoading ? (
                    <LoadingSkeleton />
                  ) : (
                    <Table
                      columns={columns}
                      data={cmsItems || []}
                      totalPages={cmsItemsTotalPages}
                      resultsPerPage={10}
                      onRowDoubleClick={handleItemView}
                      showOrdering={false}
                      searchTerm={searchTerm}
                      setSearchTerm={setSearchTerm}
                    />
                  )}
                </div>
                <div className="mb-3">
                  <Pagination
                    currentPage={currentPage}
                    setCurrentPage={setCurrentPage}
                    totalPages={cmsItemsTotalPages}
                  />
                </div>
              </div>
            )}
            {viewMode === "itemDetails" && selectedItem && selectedModel && (
              <CmsItemDetails
                cmsItem={selectedItem}
                cmsModel={selectedModel}
                onCmsItemValue={onCmsItemValue}
                onBack={handleBackToItems}
              />
            )}
            {viewMode === "itemsAssets" && selectedItem && selectedModel && (
              <CmsAssetSelector
                cmsItem={selectedItem}
                cmsModel={selectedModel}
                onAssetSelect={onCmsItemValue || (() => {})}
                onBack={handleBackToItems}
              />
            )}
          </DialogContentSection>
        </DialogContentWrapper>
      </DialogContent>
    </Dialog>
  );
};

export default CmsIntegrationDialog;
