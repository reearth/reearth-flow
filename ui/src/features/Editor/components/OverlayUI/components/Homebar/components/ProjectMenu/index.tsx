import { CaretDownIcon, PlusIcon } from "@phosphor-icons/react";
import { useNavigate } from "@tanstack/react-router";
import { memo, useState } from "react";

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
  DropdownMenuTrigger,
  ScrollArea,
} from "@flow/components";
import {
  menuGroupLabelClass,
  menuRowClass,
} from "@flow/features/common/ThemeLanguageMenu";
import { useNewWorkspace } from "@flow/features/common/WorkspaceMenu/useNewWorkspace";
import { WorkspaceMenuItems } from "@flow/features/common/WorkspaceMenu/WorkspaceMenuItems";
import {
  ProjectAddDialog,
  ProjectEditDialog,
} from "@flow/features/WorkspaceProjects/components";
import { useProject } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { usePermissions } from "@flow/lib/permissions";
import { useCurrentProject, useCurrentWorkspace } from "@flow/stores";
import type { Project, Workspace } from "@flow/types";

const ProjectMenu: React.FC = () => {
  const t = useT();
  const navigate = useNavigate();
  const [currentProject] = useCurrentProject();
  const [currentWorkspace] = useCurrentWorkspace();
  const { manageProjects } = usePermissions();
  const { useGetWorkspaceProjects } = useProject();
  const { page } = useGetWorkspaceProjects(currentWorkspace?.id);
  const { handleNewWorkspace, workspaceAddDialog } = useNewWorkspace();

  const [open, setOpen] = useState(false);
  const [openProjectAddDialog, setOpenProjectAddDialog] = useState(false);
  const [openProjectEditDialog, setOpenProjectEditDialog] = useState(false);

  const handleProjectSelect = (project: Project) => {
    if (project.id === currentProject?.id) return;
    window.location.assign(
      `/workspaces/${project.workspaceId}/projects/${project.id}`,
    );
  };

  const handleWorkspaceSelect = (workspace: Workspace) => {
    if (workspace.id === currentWorkspace?.id) return;
    navigate({ to: `/workspaces/${workspace.id}/projects` });
  };

  return (
    <>
      <DropdownMenu open={open} onOpenChange={setOpen}>
        <DropdownMenuTrigger
          className={`flex min-w-0 items-center gap-1 rounded-md px-2 py-1 hover:bg-accent ${open ? "bg-accent" : ""}`}>
          <p className="max-w-[250px] truncate text-sm">
            {currentProject?.name}
          </p>
          <CaretDownIcon
            weight="light"
            size={12}
            className="shrink-0 text-muted-foreground"
          />
        </DropdownMenuTrigger>
        <DropdownMenuContent
          className="flex w-[230px] flex-col gap-0.5"
          side="bottom"
          align="start"
          sideOffset={8}>
          <DropdownMenuLabel className={menuGroupLabelClass}>
            {t("Project")}
          </DropdownMenuLabel>
          <DropdownMenuSub>
            <DropdownMenuSubTrigger
              className={`${menuRowClass} [&>svg:last-child]:text-muted-foreground`}>
              <p className="min-w-0 flex-1 truncate">{currentProject?.name}</p>
            </DropdownMenuSubTrigger>
            <DropdownMenuSubContent className="flex w-[230px] flex-col gap-0.5 rounded-lg">
              <ScrollArea>
                <div className="flex max-h-[40vh] flex-col gap-0.5">
                  {page?.projects.map((project) => (
                    <DropdownMenuItem
                      key={project.id}
                      className={`${menuRowClass} ${project.id === currentProject?.id ? "bg-accent" : ""}`}
                      onClick={() => handleProjectSelect(project)}>
                      <p className="min-w-0 flex-1 truncate">{project.name}</p>
                    </DropdownMenuItem>
                  ))}
                </div>
              </ScrollArea>
              {manageProjects && (
                <>
                  <DropdownMenuSeparator className="mx-0 my-0" />
                  <DropdownMenuItem
                    className={`${menuRowClass} gap-2`}
                    onClick={() => setOpenProjectAddDialog(true)}>
                    <PlusIcon weight="light" />
                    <p className="flex-1 truncate">{t("New Project")}</p>
                  </DropdownMenuItem>
                </>
              )}
            </DropdownMenuSubContent>
          </DropdownMenuSub>
          {manageProjects && (
            <DropdownMenuItem
              className={menuRowClass}
              onClick={() => setOpenProjectEditDialog(true)}>
              <p className="flex-1 truncate">{t("Edit Project")}</p>
            </DropdownMenuItem>
          )}
          <DropdownMenuSeparator className="mx-0 my-0" />
          <WorkspaceMenuItems
            onNewWorkspace={handleNewWorkspace}
            onWorkspaceSelect={handleWorkspaceSelect}
          />
        </DropdownMenuContent>
      </DropdownMenu>
      {openProjectAddDialog && (
        <ProjectAddDialog
          isOpen={openProjectAddDialog}
          onOpenChange={setOpenProjectAddDialog}
        />
      )}
      {openProjectEditDialog && currentProject && (
        <ProjectEditDialog
          project={currentProject}
          onClose={() => setOpenProjectEditDialog(false)}
        />
      )}
      {workspaceAddDialog}
    </>
  );
};

export default memo(ProjectMenu);
