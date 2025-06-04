import {
  DotsThreeVertical,
  Export,
  PaperPlaneTilt,
  Question,
  SquaresFour,
} from "@phosphor-icons/react";
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
  return (
    <div className="flex shrink-0 justify-between gap-2 bg-secondary h-[44px] w-[100vw]">
      <div className="flex items-center gap-4 pl-4 pr-2">
        <FlowLogo className="size-6 transition-all " />
        <div className="flex items-center gap-2 border border-logo/50 py-0.5 px-2 rounded">
          <PaperPlaneTilt weight="thin" size={18} />
          <p className="text-accent-foreground font-light select-none">
            {t("Shared Project")}
          </p>
          <Tooltip delayDuration={0}>
            <TooltipTrigger>
              <Question weight="thin" size={16} />
            </TooltipTrigger>
            <TooltipContent className="max-w-[200px]" sideOffset={18}>
              <div className="flex flex-col gap-2">
                <div className="flex items-center gap-1">
                  <Question size={12} />
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
        <p className="flex items-center gap-2 max-w-[200px] truncate transition-all delay-0 duration-500 text-sm dark:font-thin">
          <SquaresFour weight="thin" size={18} />
          {project?.name}
        </p>
      </div>
      <div className="flex flex-1 gap-2 h-full overflow-hidden">
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
              icon={<DotsThreeVertical size={18} />}
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
              <Export weight="thin" size={18} />
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
