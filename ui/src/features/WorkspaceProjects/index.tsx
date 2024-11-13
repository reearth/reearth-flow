import { Plus } from "@phosphor-icons/react";

import { Button, FlowLogo, ScrollArea } from "@flow/components/";
import BasicBoiler from "@flow/components/BasicBoiler";
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
    currentProject,
    projectToBeDeleted,
    editProject,
    showError,
    buttonDisabled,
    openProjectAddDialog,
    setOpenProjectAddDialog,
    setEditProject,
    setProjectToBeDeleted,
    handleProjectSelect,
    handleDeleteProject,
    handleUpdateValue,
    handleUpdateProject,
  } = useHooks();

  return (
    <div className="flex h-full flex-1 flex-col">
      <div className="flex flex-1 flex-col gap-4 overflow-scroll px-6 pb-2 pt-6">
        <div className="flex items-center justify-between gap-2 border-b pb-4">
          <p className="text-lg dark:font-extralight">{t("Projects")}</p>
          <Button
            className="flex gap-2"
            variant="outline"
            onClick={() => setOpenProjectAddDialog(true)}>
            <Plus weight="thin" />
            <p className="text-xs dark:font-light">{t("New Project")}</p>
          </Button>
        </div>
        {projects && projects?.length > 0 ? (
          <ScrollArea>
            <div
              className="grid min-w-0 grid-cols-1 gap-2 overflow-scroll sm:grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 2xl:grid-cols-4"
              ref={ref}>
              {projects?.map((p) => (
                <ProjectCard
                  key={p.id}
                  project={p}
                  currentProject={currentProject}
                  setEditProject={setEditProject}
                  setProjectToBeDeleted={setProjectToBeDeleted}
                  onProjectSelect={handleProjectSelect}
                />
              ))}
            </div>
          </ScrollArea>
        ) : (
          <BasicBoiler
            text={t("No Projects")}
            icon={<FlowLogo className="size-16 text-accent" />}
          />
        )}
      </div>
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
