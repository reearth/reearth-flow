import { Plus } from "@phosphor-icons/react";
import { useNavigate } from "@tanstack/react-router";
import { useEffect } from "react";

import projectImage from "@flow/assets/project-screenshot.png"; // TODO: replace with actual project image
import {
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
} from "@flow/components";
import { useProject } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { generateWorkflows } from "@flow/mock_data/workflowData";
import { useCurrentProject, useCurrentWorkspace, useDialogType } from "@flow/stores";
import type { Project } from "@flow/types";
import { formatDate } from "@flow/utils";

const MainSection: React.FC = () => {
  const t = useT();
  const [currentWorkspace] = useCurrentWorkspace();
  const [currentProject, setCurrentProject] = useCurrentProject();
  const navigate = useNavigate({ from: "/workspace/$workspaceId" });
  const { useGetProjects, deleteProject } = useProject();
  const { projects } = useGetProjects(currentWorkspace?.id as string);
  const [, setDialogType] = useDialogType();

  const handleProjectSelect = (p: Project) => {
    if (currentWorkspace) {
      setCurrentProject(p);
      navigate({ to: `/workspace/${currentWorkspace.id}/project/${p.id}` });
    }
  };

  // TODO: Using sample workflows at the moment
  useEffect(() => {
    if (!projects) return;
    projects.forEach(p => {
      p.workflow = generateWorkflows(1)[0];
    });
  }, [projects]);

  const handleDeleteProject = async (id: string) => {
    await deleteProject(id);
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
                      {p.description && <CardDescription>{p.description}</CardDescription>}
                    </CardHeader>
                    <CardFooter className="px-3 pt-0 pb-2 flex">
                      <p className="font-thin text-xs">
                        {t("Last modified:")} {formatDate(p.updatedAt)}
                      </p>
                    </CardFooter>
                  </Card>
                </ContextMenuTrigger>
                <ContextMenuContent>
                  <ContextMenuItem>{t("Edit Details")}</ContextMenuItem>
                  <ContextMenuItem onClick={() => handleDeleteProject(p.id)}>
                    {t("Delete Project")}
                  </ContextMenuItem>
                </ContextMenuContent>
              </ContextMenu>
            ))}
          </div>
        </div>
      </div>
      <div className="">
        <p className="font-extralight text-center py-1">
          {t("Total Projects")}: {projects?.length ?? 0}
        </p>
      </div>
    </div>
  );
};

export { MainSection };
