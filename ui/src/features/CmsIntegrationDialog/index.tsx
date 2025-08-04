import { CaretLeftIcon, EyeIcon, LayoutIcon } from "@phosphor-icons/react";
import { ColumnDef } from "@tanstack/react-table";

import {
  Dialog,
  DialogContent,
  DialogContentSection,
  DialogContentWrapper,
  DialogTitle,
  Button,
  ScrollArea,
  LoadingSkeleton,
  DataTable as Table,
  FlowLogo,
  IconButton,
  Pagination,
  Input,
} from "@flow/components";
import BasicBoiler from "@flow/components/BasicBoiler";
import { useT } from "@flow/lib/i18n";
import { useCurrentWorkspace } from "@flow/stores";
import type { CmsItem } from "@flow/types/cmsIntegration";

import CmsItemDetailDialog from "./CmsItemDetailsDialog";
import CmsModelCard from "./CmsModelCard";
import CmsProjectCard from "./CmsProjectCard";
import useHooks from "./hooks";

type Props = {
  onDialogClose: () => void;
  onCmsItemDoubleClick?: (cmsItem: CmsItem) => void;
};

const CmsIntegrationDialog: React.FC<Props> = ({
  onDialogClose,
  onCmsItemDoubleClick,
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
    isItemDetailOpen,
    setSearchTerm,
    setCurrentPage,
    handleProjectSelect,
    handleModelSelect,
    handleBackToProjects,
    handleBackToModels,
    handleItemView,
    handleItemDetailClose,
  } = useHooks({
    workspaceId: currentWorkspace?.id ?? "",
  });

  const columns: ColumnDef<CmsItem>[] = selectedModel
    ? [
        {
          accessorKey: "id",
          header: "ID",
        },
        ...selectedModel.schema.fields.slice(0, 3).map((field) => ({
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
          cell: ({ row }) => (
            <div className="flex gap-1">
              <IconButton
                icon={<EyeIcon />}
                onClick={() => handleItemView(row.original)}
                title={t("View Details")}
              />
            </div>
          ),
        },
      ]
    : [];

  return (
    <Dialog open onOpenChange={onDialogClose}>
      <DialogContent className="max-h-[90vh] w-full max-w-6xl overflow-hidden">
        <DialogTitle className="flex items-center font-normal">
          <LayoutIcon size={24} className="mr-2 inline-block" />
          {t("CMS Integration")}
          {viewMode === "models" && selectedProject && (
            <div>
              <span className="mx-2 text-muted-foreground">/</span>
              <span className="text-lg">{selectedProject.name}</span>
            </div>
          )}
          {viewMode === "items" && selectedProject && selectedModel && (
            <div>
              <span className="mx-2 text-muted-foreground">/</span>
              <span className="text-lg">{selectedProject.name}</span>
              <span className="mx-2 text-muted-foreground">/</span>
              <span className="text-lg">{selectedModel.name}</span>
            </div>
          )}
        </DialogTitle>
        <DialogContentWrapper>
          <DialogContentSection className="flex h-[600px] flex-col overflow-hidden">
            <div className="mb-4 flex items-center gap-4">
              {viewMode !== "projects" && (
                <Button
                  size="icon"
                  variant="ghost"
                  onClick={
                    viewMode === "models"
                      ? handleBackToProjects
                      : handleBackToModels
                  }>
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
                      onRowDoubleClick={onCmsItemDoubleClick}
                      showOrdering={false}
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
          </DialogContentSection>
        </DialogContentWrapper>
      </DialogContent>
      {selectedItem && selectedModel && (
        <CmsItemDetailDialog
          cmsItem={selectedItem}
          cmsModel={selectedModel}
          open={isItemDetailOpen}
          onClose={handleItemDetailClose}
        />
      )}
    </Dialog>
  );
};

export default CmsIntegrationDialog;
