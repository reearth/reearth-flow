import { Plus } from "@phosphor-icons/react";
import { useNavigate } from "@tanstack/react-router";
import { useEffect, useState } from "react";

import projectImage from "@flow/assets/project-screenshot.png"; // TODO: replace with actual project image
import {
  Button,
  ButtonWithTooltip,
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuTrigger,
  Input,
  Label,
} from "@flow/components";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@flow/components/";
import { useProject } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { generateWorkflows } from "@flow/mock_data/workflowData";
import { useCurrentProject, useDialogType } from "@flow/stores";
import type { Project, Workspace } from "@flow/types";
import { formatDate } from "@flow/utils";

type Props = {
  workspace: Workspace;
};

const MainSection: React.FC<Props> = ({ workspace }) => {
  const t = useT();
  const [currentProject, setCurrentProject] = useCurrentProject();
  const navigate = useNavigate({ from: "/workspace/$workspaceId" });
  const { useGetWorkspaceProjects, deleteProject, updateProject } = useProject();
  const [, setDialogType] = useDialogType();
  const { projects } = useGetWorkspaceProjects(workspace.id);
  const [editProject, setEditProject] = useState<undefined | Project>(projects?.[0]);
  const [showError, setShowError] = useState(false);
  const [buttonDisabled, setButtonDisabled] = useState(false);

  const handleProjectSelect = (p: Project) => {
    setCurrentProject(p);
    navigate({ to: `/workspace/${workspace.id}/project/${p.id}` });
  };

  // TODO: Using sample workflows at the moment
  useEffect(() => {
    if (!projects) return;
    projects.forEach(p => {
      p.workflows = generateWorkflows(4);
    });
  }, [projects]);

  const handleDeleteProject = async (id: string) => {
    // TODO: this trigger a pop up for confirming
    await deleteProject(id, workspace.id);
  };

  const handleUpdateValue = (key: "name" | "description", value: string) => {
    if (!editProject) return;
    setEditProject({ ...editProject, [key]: value });
  };

  const handleUpdateProject = async () => {
    if (!editProject || !editProject.name) return;
    setShowError(false);
    setButtonDisabled(true);

    const { project } = await updateProject({
      projectId: editProject.id,
      name: editProject.name,
      description: editProject.description,
    });

    if (!project) {
      setShowError(true);
      setButtonDisabled(false);
      return;
    }

    setButtonDisabled(false);
    setShowError(false);
    setEditProject(undefined);
    return;
  };

  return (
    <div className="flex flex-col flex-1">
      <div className="flex flex-col flex-1 gap-8 p-8">
        <div className="flex gap-2 justify-between items-center border-b border-zinc-700 pb-4">
          <p className="text-lg font-extralight">{t("Projects")}</p>
          <ButtonWithTooltip
            className="flex gap-2 bg-zinc-800 text-zinc-300 hover:bg-zinc-700 hover:text-zinc-300"
            variant="outline"
            tooltipText={t("Create new project")}
            onClick={() => setDialogType("add-project")}>
            <Plus weight="thin" />
            <p className="text-xs font-light">{t("New Project")}</p>
          </ButtonWithTooltip>
        </div>
        <div className="flex flex-col flex-1 justify-between overflow-auto">
          <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4 overflow-auto">
            {projects?.map(p => (
              <ContextMenu key={p.id}>
                <ContextMenuTrigger>
                  <Card
                    className={`cursor-pointer bg-zinc-900/50 ${currentProject && currentProject.id === p.id ? "border-zinc-600" : "hover:border-zinc-600"}`}
                    key={p.id}
                    onClick={() => handleProjectSelect(p)}>
                    <CardContent className="p-0 rounded">
                      <img className="rounded-t-lg" src={projectImage} />
                    </CardContent>
                    <CardHeader className="p-3">
                      <CardTitle className="truncate font-extralight">{p.name}</CardTitle>
                      <CardDescription className="min-h-5">{p.description ?? ""}</CardDescription>
                    </CardHeader>
                    <CardFooter className="px-3 pt-0 pb-2 flex">
                      <p className="font-thin text-xs">
                        {t("Last modified:")} {formatDate(p.updatedAt)}
                      </p>
                    </CardFooter>
                  </Card>
                </ContextMenuTrigger>
                <ContextMenuContent>
                  <ContextMenuItem onClick={() => setEditProject({ ...p })}>
                    {t("Edit Details")}
                  </ContextMenuItem>
                  <ContextMenuItem onClick={() => handleDeleteProject(p.id)}>
                    {t("Delete Project")}
                  </ContextMenuItem>
                </ContextMenuContent>
              </ContextMenu>
            ))}
          </div>
        </div>
      </div>
      <Dialog open={!!editProject}>
        <DialogContent hideCloseButton={true}>
          <DialogHeader>
            <DialogTitle>{t("Edit Project")}</DialogTitle>
            <DialogDescription className="px-6">
              <div className="flex flex-col gap-4 mt-4">
                <div className="flex flex-col gap-2">
                  <Label>{t("Project Name: ")}</Label>
                  <Input
                    value={editProject?.name}
                    onChange={e => handleUpdateValue("name", e.target.value)}
                  />
                </div>
                <div className="flex flex-col gap-2">
                  <Label>{t("Project Description: ")}</Label>
                  <Input
                    placeholder={t("Project Description")}
                    value={editProject?.description}
                    onChange={e => handleUpdateValue("description", e.target.value)}
                  />
                </div>
              </div>
              <div
                className={`text-xs text-red-400 mt-2 ${showError ? "opacity-70" : "opacity-0"}`}>
                {t("Failed to update project")}
              </div>
            </DialogDescription>

            <div className="px-6 pb-6 flex gap-4 justify-end">
              <Button disabled={buttonDisabled || !editProject?.name} onClick={handleUpdateProject}>
                {t("Save")}
              </Button>
              <Button
                disabled={buttonDisabled}
                variant={"outline"}
                onClick={() => setEditProject(undefined)}>
                {t("Cancel")}
              </Button>
            </div>
          </DialogHeader>
        </DialogContent>
      </Dialog>
      <div>
        <p className="font-extralight text-center py-1">
          {t("Total Projects")}: {projects?.length ?? 0}
        </p>
      </div>
    </div>
  );
};

export { MainSection };
