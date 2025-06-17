import {
  DotsThreeVerticalIcon,
  ExportIcon,
  PaperPlaneTiltIcon,
  QuestionIcon,
  SquaresFourIcon,
} from "@phosphor-icons/react";
import { useNavigate } from "@tanstack/react-router";
import { memo } from "react";
import { Doc } from "yjs";

import {
  Button,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  FlowLogo,
  IconButton,
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@flow/components";
import { WorkflowTabs } from "@flow/features/Editor/components";
import { useT } from "@flow/lib/i18n";
import type { Me, Project, Workspace } from "@flow/types";

import ImportDialog from "../ImportDialog";

import useHooks from "./hooks";

type Props = {
  currentWorkflowId: string;
  openWorkflows: {
    id: string;
    name: string;
  }[];
  yDoc: Doc | null;
  project?: Project;
  accessToken?: string;
  me?: Me;
  workspaces?: Workspace[] | undefined;
  onWorkflowClose: (workflowId: string) => void;
  onWorkflowChange: (workflowId?: string) => void;
};

const SharedCanvasTopBar: React.FC<Props> = ({
  currentWorkflowId,
  openWorkflows,
  yDoc,
  project,
  accessToken,
  me,
  workspaces,
  onWorkflowClose,
  onWorkflowChange,
}) => {
  const {
    showDialog,
    selectedWorkspace,
    handleSelectWorkspace,
    handleSharedProjectExport,
    handleSharedProjectImport,
    handleDialogClose,
    handleShowImportDialog,
  } = useHooks({ yDoc, project, accessToken });
  const t = useT();

  const navigate = useNavigate();

  return (
    <div className="flex h-[44px] w-[100vw] shrink-0 justify-between gap-2 bg-secondary">
      <div className="flex items-center gap-4 pr-2 pl-4">
        <div onClick={() => navigate({ to: "/" })}>
          <FlowLogo className="size-6 cursor-pointer transition-all" />
        </div>
        <div className="flex items-center gap-2 rounded border border-logo/50 px-2 py-0.5">
          <PaperPlaneTiltIcon weight="thin" size={18} />
          <p className="font-light text-accent-foreground select-none">
            {t("Shared Project")}
          </p>
          <Tooltip delayDuration={0}>
            <TooltipTrigger>
              <QuestionIcon weight="thin" size={16} />
            </TooltipTrigger>
            <TooltipContent className="max-w-[200px]" sideOffset={18}>
              <div className="flex flex-col gap-2">
                <div className="flex items-center gap-1">
                  <QuestionIcon size={12} />
                  <p>{t("Shared project")}</p>
                </div>
                <p>
                  {t(
                    "A shared project is in a read only state. To start editing or to run this project, please import it into one of your workspaces.",
                  )}
                </p>
              </div>
            </TooltipContent>
          </Tooltip>
        </div>
      </div>
      <div className="flex items-center pr-4 pl-2">
        <p className="flex max-w-[200px] items-center gap-2 truncate text-sm transition-all delay-0 duration-500 dark:font-thin">
          <SquaresFourIcon weight="thin" size={18} />
          {project?.name}
        </p>
      </div>
      <div className="flex h-full flex-1 gap-2 overflow-hidden">
        <WorkflowTabs
          currentWorkflowId={currentWorkflowId}
          openWorkflows={openWorkflows}
          onWorkflowClose={onWorkflowClose}
          onWorkflowChange={onWorkflowChange}
        />
      </div>
      <div className="flex items-center justify-center gap-2 p-1">
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <IconButton
              className="w-[25px]"
              tooltipText={t("Additional actions")}
              tooltipOffset={6}
              icon={<DotsThreeVerticalIcon size={18} />}
            />
          </DropdownMenuTrigger>
          <DropdownMenuContent
            className="flex flex-col gap-2"
            align="start"
            sideOffset={10}
            alignOffset={2}>
            <DropdownMenuItem
              className="flex justify-between gap-4"
              onClick={handleSharedProjectExport}>
              <p>{t("Export Project")}</p>
              <ExportIcon weight="thin" size={18} />
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
        <Button
          type="button"
          variant="outline"
          size="sm"
          disabled={!me}
          onClick={handleShowImportDialog}>
          {t("Import into Workspace")}
        </Button>
        {showDialog === "import" && workspaces && (
          <ImportDialog
            workspaces={workspaces}
            selectedWorkspace={selectedWorkspace}
            onSelectWorkspace={handleSelectWorkspace}
            onImportProject={handleSharedProjectImport}
            onDialogClose={handleDialogClose}
          />
        )}
      </div>
    </div>
  );
};

export default memo(SharedCanvasTopBar);
