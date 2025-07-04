import {
  ClipboardTextIcon,
  CopyIcon,
  DotsThreeVerticalIcon,
  ExportIcon,
  PencilSimpleIcon,
  PaperPlaneTiltIcon,
  TrashIcon,
} from "@phosphor-icons/react";
import { MouseEvent, useState } from "react";

import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
  FlowLogo,
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@flow/components";
import { useToast } from "@flow/features/NotificationSystem/useToast";
import { useT } from "@flow/lib/i18n";
import { Project } from "@flow/types";
import { openLinkInNewTab } from "@flow/utils";
import { copyToClipboard } from "@flow/utils/copyToClipboard";

import useProjectExportFromCard from "./useProjectExportFromCard";

type Props = {
  project: Project;
  isDuplicating: boolean;
  setEditProject: (project: Project | undefined) => void;
  setDuplicateProject: (project: Project | undefined) => void;
  setProjectToBeDeleted: (project: string | undefined) => void;
  onProjectSelect: (p: Project) => void;
};

const ProjectCard: React.FC<Props> = ({
  project,
  isDuplicating,
  setEditProject,
  setDuplicateProject,
  setProjectToBeDeleted,
  onProjectSelect,
}) => {
  const t = useT();
  const { toast } = useToast();
  const { id, name, description, updatedAt, sharedToken } = project;
  const { handleProjectExportFromCard, isExporting } =
    useProjectExportFromCard(project);
  const [persistOverlay, setPersistOverlay] = useState(false);
  // TODO: isShared and sharedURL are temp values.

  const BASE_URL = window.location.origin;

  const sharedUrl = sharedToken
    ? BASE_URL + "/shared/" + sharedToken
    : undefined;

  const handleCopyURLToClipBoard = () => {
    if (!sharedUrl) return;
    copyToClipboard(sharedUrl);
    toast({
      title: t("Copied to clipboard"),
      description: t("{{project}} project's share URL copied to clipboard", {
        project: name,
      }),
    });
  };

  const handleOpenSharedProject = (e: MouseEvent) => {
    if (!sharedUrl) return;
    e.stopPropagation();
    openLinkInNewTab(sharedUrl);
  };

  return (
    <Card
      className="group relative cursor-pointer border-transparent bg-secondary hover:border-border"
      key={id}
      onClick={() => onProjectSelect(project)}>
      <CardContent className="relative flex h-[120px] items-center justify-center p-0">
        {isExporting && (
          <p className="loading-pulse absolute top-2 left-2 font-thin">
            {t("Exporting...")}
          </p>
        )}
        {isDuplicating && (
          <p className="loading-pulse absolute top-2 left-2 font-thin">
            {t("Duplicating...")}
          </p>
        )}
        <FlowLogo
          className={`size-[120px] translate-x-20 opacity-50 ${description ? "group:hover:opacity-90" : ""}`}
        />
      </CardContent>
      <CardHeader className="px-2 py-1">
        <CardTitle className="truncate dark:font-extralight">{name}</CardTitle>
      </CardHeader>
      <CardFooter className="flex px-2 pb-1">
        <p className="text-xs text-zinc-400 dark:font-thin">
          {t("Last modified:")} {updatedAt}
        </p>
      </CardFooter>
      <div
        className={`absolute inset-0 ${persistOverlay ? "flex flex-col" : "hidden"} rounded-lg group-hover:flex group-hover:flex-col`}>
        <div
          className={`flex h-[120px] items-center justify-center rounded-t-lg bg-black/30 p-4 ${description ? "backdrop-blur-xs" : ""}`}>
          <p className="line-clamp-4 overflow-hidden text-center text-sm break-words text-ellipsis whitespace-normal text-secondary dark:font-light dark:text-foreground">
            {description}
          </p>
        </div>
        <div className="flex flex-1 justify-end rounded-b-lg">
          <DropdownMenu
            modal={false}
            onOpenChange={(o) => setPersistOverlay(o)}>
            <DropdownMenuTrigger
              className="flex h-full w-[30px] items-center justify-center rounded-br-lg hover:bg-primary"
              onClick={(e) => e.stopPropagation()}>
              <DotsThreeVerticalIcon className="size-[24px]" />
            </DropdownMenuTrigger>
            <DropdownMenuContent
              align="end"
              onClick={(e) => e.stopPropagation()}>
              <DropdownMenuItem
                className="justify-between gap-2 text-warning"
                onClick={() => setEditProject({ ...project })}>
                {t("Edit Details")}
                <PencilSimpleIcon />
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem
                className="justify-between gap-2"
                onClick={handleProjectExportFromCard}>
                {t("Export Project")}
                <ExportIcon weight="light" />
              </DropdownMenuItem>
              <DropdownMenuItem
                className="justify-between gap-2"
                onClick={() => setDuplicateProject({ ...project })}>
                {t("Duplicate Project")}
                <CopyIcon weight="light" />
              </DropdownMenuItem>
              <DropdownMenuItem
                className="justify-between gap-2"
                disabled={!sharedUrl}
                onClick={handleCopyURLToClipBoard}>
                {t("Copy Share URL")}
                <ClipboardTextIcon weight="light" />
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem
                className="justify-between gap-4 text-destructive"
                onClick={(e) => {
                  e.stopPropagation();
                  setProjectToBeDeleted(id);
                }}>
                {t("Delete Project")}
                <TrashIcon weight="light" />
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </div>
      {sharedUrl && (
        <Tooltip>
          {/* <TooltipTrigger className="absolute right-1 top-1 rounded p-1 text-muted-foreground hover:bg-primary group-hover:text-white"> */}
          <TooltipTrigger
            className="absolute top-1 right-1 rounded p-1 text-muted-foreground group-hover:text-white hover:bg-primary"
            onClick={handleOpenSharedProject}>
            <PaperPlaneTiltIcon />
          </TooltipTrigger>
          <TooltipContent>{t("Public Read Access")}</TooltipContent>
        </Tooltip>
      )}
    </Card>
  );
};

export { ProjectCard };
