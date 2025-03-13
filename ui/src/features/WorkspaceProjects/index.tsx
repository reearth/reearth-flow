import { ArrowSquareIn, CaretDown, Plus } from "@phosphor-icons/react";

import {
  Button,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuTrigger,
  FlowLogo,
  LoadingSkeleton,
  Pagination,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@flow/components";
import BasicBoiler from "@flow/components/BasicBoiler";
import {
  ALLOWED_PROJECT_IMPORT_EXTENSIONS,
  ALLOWED_WORKFLOW_FILE_EXTENSIONS,
} from "@flow/global-constants";
import { useProjectImport, useWorkflowImport } from "@flow/hooks";
import { useT } from "@flow/lib/i18n";

import {
  ProjectAddDialog,
  ProjectCard,
  ProjectDeletionDialog,
  ProjectEditDialog,
} from "./components";
import useHooks from "./hooks";

const ProjectsManager: React.FC = () => {
  const t = useT();

  const {
    projects,
    ref,
    projectToBeDeleted,
    editProject,
    showError,
    buttonDisabled,
    openProjectAddDialog,
    currentPage,
    totalPages,
    isFetching,
    isDuplicating,
    currentOrder,
    orderDirections,
    setOpenProjectAddDialog,
    setEditProject,
    setProjectToBeDeleted,
    setCurrentPage,
    handleProjectDuplication,
    handleProjectSelect,
    handleDeleteProject,
    handleUpdateValue,
    handleUpdateProject,
    handleOrderChange,
  } = useHooks();

  const {
    fileInputRef: fileInputRefProject,
    isProjectImporting,
    handleProjectImportClick,
    handleProjectFileUpload,
  } = useProjectImport();

  const {
    fileInputRef: fileInputRefWorkflow,
    // isWorkflowImporting,
    // invalidFile,
    // setIsWorkflowImporting,
    handleWorkflowImportClick,
    handleWorkflowFileUpload,
  } = useWorkflowImport();
  return (
    <div className="flex h-full flex-1 flex-col">
      <div className="flex flex-1 flex-col gap-4 overflow-scroll px-6 pb-2 pt-4">
        <div className="flex h-[50px] items-center justify-between gap-2 border-b pb-4">
          <p className="text-lg dark:font-extralight">{t("Projects")}</p>
          <div className="flex gap-2">
            <DropdownMenu>
              <DropdownMenuTrigger className="flex items-center gap-1 rounded-md p-2 hover:bg-primary">
                <ArrowSquareIn weight="thin" />
                <p className="line-clamp-2 text-xs font-extralight">
                  {t("Import")}
                </p>
                <div className="shrink-0">
                  <CaretDown size="12px" weight="thin" />
                </div>
              </DropdownMenuTrigger>
              <DropdownMenuContent>
                <DropdownMenuGroup>
                  <DropdownMenuItem onClick={handleProjectImportClick}>
                    <p className="text-sm">
                      {t("Project ")}
                      <span className="font-thin">(flow.zip)</span>
                    </p>
                  </DropdownMenuItem>
                  <DropdownMenuItem onClick={handleWorkflowImportClick}>
                    <p className="text-sm">
                      {t("Workflow ")}
                      <span className="font-thin">(yaml or json)</span>
                    </p>
                  </DropdownMenuItem>
                </DropdownMenuGroup>
              </DropdownMenuContent>
            </DropdownMenu>
            <Button
              className="flex gap-2"
              variant="default"
              onClick={() => setOpenProjectAddDialog(true)}>
              <Plus weight="thin" />
              <p className="text-xs dark:font-light">{t("New Project")}</p>
            </Button>
          </div>
        </div>
        {currentOrder && (
          <Select
            value={currentOrder || "DESC"}
            onValueChange={handleOrderChange}>
            <SelectTrigger className="w-[100px]">
              <SelectValue placeholder={orderDirections.ASC} />
            </SelectTrigger>
            <SelectContent>
              {Object.entries(orderDirections).map(([value, label]) => (
                <SelectItem key={value} value={value}>
                  {label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        )}
        {isFetching || isProjectImporting || isDuplicating ? (
          <LoadingSkeleton />
        ) : projects && projects.length > 0 ? (
          <div
            className="grid min-w-0 grid-cols-1 gap-2 overflow-scroll sm:grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 2xl:grid-cols-4"
            ref={ref}>
            {projects?.map((p) => (
              <ProjectCard
                key={p.id}
                project={p}
                isDuplicating={isDuplicating}
                setEditProject={setEditProject}
                setProjectToBeDeleted={setProjectToBeDeleted}
                onProjectSelect={handleProjectSelect}
                onProjectDuplication={handleProjectDuplication}
              />
            ))}
          </div>
        ) : (
          <BasicBoiler
            text={t("No Projects")}
            icon={<FlowLogo className="size-16 text-accent" />}
          />
        )}
      </div>
      <div className="mb-3">
        <Pagination
          currentPage={currentPage}
          setCurrentPage={setCurrentPage}
          totalPages={totalPages}
        />
      </div>
      {/* This (ghost) input is used for uploading the project to be imported */}
      <input
        type="file"
        accept={ALLOWED_PROJECT_IMPORT_EXTENSIONS}
        ref={fileInputRefProject}
        onChange={handleProjectFileUpload}
        style={{ display: "none" }}
      />
      <input
        type="file"
        accept={ALLOWED_WORKFLOW_FILE_EXTENSIONS}
        ref={fileInputRefWorkflow}
        onChange={handleWorkflowFileUpload}
        style={{ display: "none" }}
      />
      <ProjectAddDialog
        isOpen={openProjectAddDialog}
        onOpenChange={(o) => setOpenProjectAddDialog(o)}
      />
      <ProjectEditDialog
        editProject={editProject}
        showError={showError}
        buttonDisabled={buttonDisabled}
        setEditProject={setEditProject}
        onUpdateValue={handleUpdateValue}
        onUpdateProject={handleUpdateProject}
      />
      <ProjectDeletionDialog
        projectToBeDeleted={projectToBeDeleted}
        setProjectToBeDeleted={setProjectToBeDeleted}
        onDeleteProject={handleDeleteProject}
      />
    </div>
  );
};

export default ProjectsManager;
