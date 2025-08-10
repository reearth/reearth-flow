import { CaretLeftIcon, EyeIcon } from "@phosphor-icons/react";
import { ColumnDef } from "@tanstack/react-table";

import {
  Dialog,
  DialogContent,
  DialogContentSection,
  DialogContentWrapper,
  DialogTitle,
  ScrollArea,
  DataTable as Table,
  FlowLogo,
  IconButton,
  Pagination,
  Input,
  Button,
  Skeleton,
  LoadingTableSkeleton,
} from "@flow/components";
import BasicBoiler from "@flow/components/BasicBoiler";
import { CMS_ITEMS_FETCH_RATE } from "@flow/lib/gql/cms/useQueries";
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
    isDebouncing,
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
                      Array.from({ length: 12 }).map((_, index) => (
                        <div
                          key={index}
                          className="flex h-[140px] flex-col justify-between rounded-lg bg-secondary">
                          <div className="flex h-[50px] w-[200px] flex-col justify-center gap-1 px-2">
                            <div className="flex items-center gap-2 truncate text-base">
                              <Skeleton className=" h-[20px] w-[150px]" />
                              <Skeleton className=" h-[20px] w-[66px] rounded-full" />
                            </div>
                            <Skeleton className="h-[16px] w-[165px]" />
                          </div>
                          <div className="flex justify-between px-2 pb-2">
                            <Skeleton className="mb-2 h-[16px] w-[120px]" />
                            <Skeleton className="mb-2 h-[16px] w-[100px]" />
                          </div>
                        </div>
                      ))
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
                      Array.from({ length: 12 }).map((_, index) => (
                        <div
                          key={index}
                          className="flex h-[108px] flex-col justify-between rounded-lg bg-secondary">
                          <div className="flex h-[44px] w-[200px] flex-col justify-center gap-1 px-2">
                            <Skeleton className=" h-[20px] w-[150px]" />
                          </div>
                          <div className="flex justify-between px-2 pb-2">
                            <Skeleton className="mb-2 h-[16px] w-[66px]" />
                            <Skeleton className="mb-2 h-[16px] w-[100px]" />
                            <Skeleton className="mb-2 h-[16px] w-[100px]" />
                          </div>
                        </div>
                      ))
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
                {isDebouncing || isLoading ? (
                  <LoadingTableSkeleton
                    columns={columns.length}
                    rows={CMS_ITEMS_FETCH_RATE}
                    hasQuickActions
                  />
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
                    searchTerm={searchTerm}
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
