import projectImage from "@flow/assets/project-screenshot.png"; // TODO: replace with actual project image
import {
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
import { useT } from "@flow/lib/i18n";
import { Project } from "@flow/types";
import { formatDate } from "@flow/utils";

type Props = {
  project: Project;
  currentProject: Project | undefined;
  setEditProject: (project: Project | undefined) => void;
  setProjectToBeDeleted: (project: string | undefined) => void;
  onProjectSelect: (p: Project) => void;
};

const ProjectCard: React.FC<Props> = ({
  project,
  currentProject,
  setEditProject,
  setProjectToBeDeleted,
  onProjectSelect,
}) => {
  const t = useT();
  return (
    <ContextMenu key={project.id}>
      <ContextMenuTrigger>
        <Card
          className={`cursor-pointer border-transparent bg-secondary ${currentProject && currentProject.id === project.id ? "border-border" : "hover:border-border"}`}
          key={project.id}
          onClick={() => onProjectSelect(project)}
        >
          <CardContent className="rounded p-0">
            <img className="rounded-t-lg" src={projectImage} />
          </CardContent>
          <CardHeader className="p-3">
            <CardTitle className="truncate font-extralight">
              {project.name}
            </CardTitle>
            <CardDescription className="min-h-5">
              {project.description ?? ""}
            </CardDescription>
          </CardHeader>
          <CardFooter className="flex px-3 pb-2 pt-0">
            <p className="text-xs font-thin">
              {t("Last modified:")} {formatDate(project.updatedAt)}
            </p>
          </CardFooter>
        </Card>
      </ContextMenuTrigger>
      <ContextMenuContent>
        <ContextMenuItem onClick={() => setEditProject({ ...project })}>
          {t("Edit Details")}
        </ContextMenuItem>
        <ContextMenuItem onClick={() => setProjectToBeDeleted(project.id)}>
          {t("Delete Project")}
        </ContextMenuItem>
      </ContextMenuContent>
    </ContextMenu>
  );
};

export { ProjectCard };
