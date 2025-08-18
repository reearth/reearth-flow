import { CaretLeftIcon, EyeIcon } from "@phosphor-icons/react";
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
    cmsModels,
    cmsModelsTotalPages,
    cmsItems,
    cmsItemsTotalPages,
    itemsCurrentPage,
    modelsCurrentPage,
    searchTerm,
    isLoading,
    viewMode,
    setSearchTerm,
    setModelsCurrentPage,
    setItemsCurrentPage,
    handleProjectSelect,
    handleModelSelect,
    handleBackToProjects,
    handleBackToModels,
    handleItemView,
    handleBackToItems,
  } = useHooks({
    workspaceId: currentWorkspace?.id ?? "",
  });

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
            return (
              <div className="flex gap-1">
                <IconButton
                  icon={<EyeIcon />}
                  onClick={() => handleItemView(row.original)}
                  title={t("View Details")}
                />
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
            onBackToModels={handleBackToModels}
            onBackToItems={handleBackToItems}
          />
        </DialogTitle>
        <DialogContentWrapper>
          <DialogContentSection className="flex h-[600px] flex-col overflow-hidden">
            <div className="mb-3 flex items-center gap-2">
              {viewMode !== "projects" && (
                <Button
                  className="p-2.5"
                  onClick={
                    viewMode === "models"
                      ? handleBackToProjects
                      : viewMode === "items"
                        ? handleBackToModels
                        : handleBackToItems
                  }
                  variant="ghost">
                  <CaretLeftIcon />
                </Button>
              )}
              {viewMode !== "itemDetails" &&
                viewMode !== "models" &&
                viewMode !== "projects" && (
                  <Input
                    placeholder={t("Search") + "..."}
                    value={searchTerm ?? ""}
                    onChange={(e) => setSearchTerm(e.target.value)}
                    className="h-[36px] max-w-sm"
                  />
                )}
            </div>
            <ScrollArea
              className={`${viewMode === "items" ? "hidden" : "flex-1"}`}>
              <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
                {viewMode === "projects" && (
                  <>
                    {isLoading ? (
                      <div className="col-span-full py-8 pt-50 text-muted-foreground">
                        <LoadingSkeleton />
                      </div>
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
                  </>
                )}
                {viewMode === "models" && selectedProject && (
                  <>
                    {isLoading ? (
                      <div className="col-span-full py-8 pt-50 text-center text-muted-foreground">
                        <LoadingSkeleton />
                      </div>
                    ) : cmsModels.length === 0 ? (
                      <div className="col-span-full py-8 text-center text-muted-foreground">
                        <BasicBoiler
                          text={t("No Models Found")}
                          className="[&>div>p]:text-md size-4 pt-60"
                          icon={<FlowLogo className="size-14 text-accent" />}
                        />
                      </div>
                    ) : (
                      cmsModels.map((model) => {
                        return (
                          <CmsModelCard
                            key={model.id}
                            model={model}
                            onModelSelect={handleModelSelect}
                          />
                        );
                      })
                    )}
                  </>
                )}
              </div>
            </ScrollArea>
            {viewMode === "items" && selectedModel && (
              <div className="h-full flex-1 overflow-hidden">
                {isLoading ? (
                  <LoadingSkeleton />
                ) : cmsItems.length === 0 ? (
                  <div className="col-span-full py-8 text-center text-muted-foreground">
                    <BasicBoiler
                      text={t("No Items Found")}
                      className="[&>div>p]:text-md size-4 pt-60"
                      icon={<FlowLogo className="size-14 text-accent" />}
                    />
                  </div>
                ) : (
                  <Table
                    columns={columns}
                    data={cmsItems || []}
                    totalPages={cmsItemsTotalPages}
                    resultsPerPage={10}
                    onRowDoubleClick={handleItemView}
                    showOrdering={false}
                    setSearchTerm={setSearchTerm}
                  />
                )}
              </div>
            )}
            {viewMode !== "projects" && viewMode !== "itemDetails" && (
              <div className="mb-3">
                <Pagination
                  currentPage={
                    viewMode === "models" ? modelsCurrentPage : itemsCurrentPage
                  }
                  setCurrentPage={
                    viewMode === "models"
                      ? setModelsCurrentPage
                      : setItemsCurrentPage
                  }
                  totalPages={cmsModelsTotalPages}
                />
              </div>
            )}
            {viewMode === "itemDetails" && selectedItem && selectedModel && (
              <CmsItemDetails
                cmsItem={selectedItem}
                cmsModel={selectedModel}
                onCmsItemValue={onCmsItemValue}
              />
            )}
          </DialogContentSection>
        </DialogContentWrapper>
      </DialogContent>
    </Dialog>
  );
};

export default CmsIntegrationDialog;
