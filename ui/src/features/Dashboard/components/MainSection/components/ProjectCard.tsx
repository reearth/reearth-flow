import { DotsThreeVertical } from "@phosphor-icons/react";
import { useState } from "react";

import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  FlowLogo,
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
  const { id, name, description, updatedAt } = project;

  const [persistOverlay, setPersistOverlay] = useState(false);

  return (
    <Card
      className={`group relative cursor-pointer border-transparent bg-secondary ${currentProject && currentProject.id === id ? "border-border" : "hover:border-border"}`}
      key={id}
      onClick={() => onProjectSelect(project)}
    >
      <CardContent className="flex h-[120px] items-center justify-center rounded-t-lg bg-red-800/10 p-0">
        <FlowLogo
          className={`size-[40px] text-zinc-300 ${description ? "group:hover:opacity-90" : ""}`}
        />
      </CardContent>
      <CardHeader className="px-2 py-1">
        <CardTitle className="truncate font-extralight">{name}</CardTitle>
      </CardHeader>
      <CardFooter className="flex px-2 pb-1">
        <p className="text-xs font-thin text-zinc-400">
          {t("Last modified:")} {formatDate(updatedAt)}
        </p>
      </CardFooter>
      <div
        className={`absolute inset-0 ${persistOverlay ? "flex flex-col" : "hidden"} rounded-lg group-hover:flex group-hover:flex-col`}
      >
        <div
          className={`flex h-[120px] items-center justify-center rounded-t-lg bg-black/30 p-4 ${description ? "backdrop-blur-sm" : ""}`}
        >
          <p className="line-clamp-4 overflow-hidden text-ellipsis whitespace-normal break-words text-center text-sm font-light text-zinc-300">
            {description}
          </p>
        </div>
        <div className="flex flex-1 justify-end rounded-b-lg">
          <DropdownMenu
            modal={false}
            onOpenChange={(o) => setPersistOverlay(o)}
          >
            <DropdownMenuTrigger
              className="flex h-full w-[30px] items-center justify-center rounded-br-lg text-zinc-400 hover:bg-zinc-800"
              onClick={(e) => e.stopPropagation()}
            >
              <DotsThreeVertical className="size-[24px]" />
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              <DropdownMenuItem
                onClick={(e) => {
                  e.stopPropagation();
                  setEditProject({ ...project });
                }}
              >
                {t("Edit Details")}
              </DropdownMenuItem>
              <DropdownMenuItem
                onClick={(e) => {
                  e.stopPropagation();
                  setProjectToBeDeleted(id);
                }}
              >
                {t("Delete Project")}
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </div>
    </Card>
  );
};

export { ProjectCard };
