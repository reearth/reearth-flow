import { Plus } from "@phosphor-icons/react";
import { useNavigate } from "@tanstack/react-router";
import { useEffect, useMemo, useRef, useState } from "react";

import projectImage from "@flow/assets/project-screenshot.png"; // TODO: replace with actual project image
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  Button,
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
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  Input,
  Label,
} from "@flow/components/";
import { useProject } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { useCurrentProject, useDialogType } from "@flow/stores";
import type { Project, Workspace } from "@flow/types";
import { formatDate } from "@flow/utils";

type Props = {
  workspace: Workspace;
};

const MainSection: React.FC<Props> = ({ workspace }) => {
  const t = useT();
  const ref = useRef<HTMLDivElement>(null);
  const [currentProject, setCurrentProject] = useCurrentProject();
  const navigate = useNavigate({ from: "/workspace/$workspaceId" });
  const { useGetWorkspaceProjectsInfinite, deleteProject, updateProject } =
    useProject();
  const [, setDialogType] = useDialogType();
  const { pages, hasNextPage, isFetching, fetchNextPage } =
    useGetWorkspaceProjectsInfinite(workspace.id);
  const [showError, setShowError] = useState(false);
  const [buttonDisabled, setButtonDisabled] = useState(false);

  const [projectToBeDeleted, setProjectToBeDeleted] = useState<
    string | undefined
  >(undefined);
  const [editProject, setEditProject] = useState<undefined | Project>(
    undefined
  );

  const handleProjectSelect = (p: Project) => {
    setCurrentProject(p);
    navigate({ to: `/workspace/${workspace.id}/project/${p.id}` });
  };

  const handleDeleteProject = async (id: string) => {
    setProjectToBeDeleted(undefined);
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

  const projects: Project[] | undefined = useMemo(
    () =>
      pages?.reduce((projects, page) => {
        if (page?.projects) {
          projects.push(...page.projects);
        }
        return projects;
      }, [] as Project[]),
    [pages]
  );
  // Auto fills the page
  useEffect(() => {
    if (
      ref.current &&
      ref.current?.scrollHeight <= document.documentElement.clientHeight &&
      hasNextPage &&
      !isFetching
    ) {
      fetchNextPage();
    }
  }, [isFetching, hasNextPage, ref, fetchNextPage]);

  // Loads more projects as scroll reaches the bottom
  useEffect(() => {
    const handleScroll = () => {
      if (
        window.innerHeight + document.documentElement.scrollTop + 5 >=
          document.documentElement.scrollHeight &&
        !isFetching &&
        hasNextPage
      ) {
        fetchNextPage();
      }
    };
    window.addEventListener("scroll", handleScroll);
    return () => window.removeEventListener("scroll", handleScroll);
  }, [isFetching, fetchNextPage, hasNextPage]);

  return (
    <div className="flex flex-1 flex-col">
      <div className="flex flex-1 flex-col gap-8 p-8">
        <div className="flex items-center justify-between gap-2 border-b pb-4">
          <p className="text-lg font-extralight">{t("Projects")}</p>
          <Button
            className="flex gap-2"
            variant="outline"
            onClick={() => setDialogType("add-project")}
          >
            <Plus weight="thin" />
            <p className="text-xs font-light">{t("New Project")}</p>
          </Button>
        </div>
        <div className="flex flex-1 flex-col justify-between overflow-auto">
          <div
            className="grid grid-cols-1 gap-4 overflow-auto sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5"
            ref={ref}
          >
            {projects?.map((p) => (
              <ContextMenu key={p.id}>
                <ContextMenuTrigger>
                  <Card
                    className={`cursor-pointer border-transparent bg-secondary ${currentProject && currentProject.id === p.id ? "border-border" : "hover:border-border"}`}
                    key={p.id}
                    onClick={() => handleProjectSelect(p)}
                  >
                    <CardContent className="rounded p-0">
                      <img className="rounded-t-lg" src={projectImage} />
                    </CardContent>
                    <CardHeader className="p-3">
                      <CardTitle className="truncate font-extralight">
                        {p.name}
                      </CardTitle>
                      <CardDescription className="min-h-5">
                        {p.description ?? ""}
                      </CardDescription>
                    </CardHeader>
                    <CardFooter className="flex px-3 pb-2 pt-0">
                      <p className="text-xs font-thin">
                        {t("Last modified:")} {formatDate(p.updatedAt)}
                      </p>
                    </CardFooter>
                  </Card>
                </ContextMenuTrigger>
                <ContextMenuContent>
                  <ContextMenuItem onClick={() => setEditProject({ ...p })}>
                    {t("Edit Details")}
                  </ContextMenuItem>
                  <ContextMenuItem onClick={() => setProjectToBeDeleted(p.id)}>
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
              <div className="mt-4 flex flex-col gap-4">
                <div className="flex flex-col gap-2">
                  <Label>{t("Project Name")}</Label>
                  <Input
                    value={editProject?.name}
                    placeholder={t("Project Name")}
                    onChange={(e) => handleUpdateValue("name", e.target.value)}
                  />
                </div>
                <div className="flex flex-col gap-2">
                  <Label>{t("Project Description")}</Label>
                  <Input
                    placeholder={t("Project Description")}
                    value={editProject?.description}
                    onChange={(e) =>
                      handleUpdateValue("description", e.target.value)
                    }
                  />
                </div>
              </div>
              <div
                className={`mt-2 text-xs text-red-400 ${showError ? "opacity-70" : "opacity-0"}`}
              >
                {t("Failed to update project")}
              </div>
            </DialogDescription>

            <div className="flex justify-end gap-4 px-6 pb-6">
              <Button
                disabled={buttonDisabled}
                variant={"outline"}
                onClick={() => setEditProject(undefined)}
              >
                {t("Cancel")}
              </Button>
              <Button
                disabled={buttonDisabled || !editProject?.name}
                onClick={handleUpdateProject}
              >
                {t("Save")}
              </Button>
            </div>
          </DialogHeader>
        </DialogContent>
      </Dialog>
      <AlertDialog open={!!projectToBeDeleted}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t("Are you absolutely sure?")}</AlertDialogTitle>
            <AlertDialogDescription>
              {t(
                "This action cannot be undone. This will permanently delete your project and remove your data from our servers."
              )}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel onClick={() => setProjectToBeDeleted(undefined)}>
              Cancel
            </AlertDialogCancel>
            <AlertDialogAction
              disabled={!projectToBeDeleted}
              onClick={() =>
                projectToBeDeleted && handleDeleteProject(projectToBeDeleted)
              }
            >
              Continue
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
};

export { MainSection };
