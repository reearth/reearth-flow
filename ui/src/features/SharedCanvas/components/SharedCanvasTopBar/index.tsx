import {
  ArrowSquareIn,
  DotsThreeVertical,
  Export,
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
      <div className="flex items-center gap-1">
        <div className="self-start h-full flex gap-2 items-center pl-4 pr-2 group">
          <FlowLogo className="size-6 transition-all" />
        </div>
      </div>
      <div className="flex items-center pr-4 pl-2">
        <div className="flex cursor-default select-none items-center gap-2">
          <p className="max-w-[200px] truncate transition-all delay-0 duration-500 text-sm dark:font-thin">
            {project?.name}
          </p>
        </div>
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
          variant="default"
          size="sm"
          disabled={!me}
          onClick={handleShowImportDialog}>
          {t("Import into Workspace")}
          <ArrowSquareIn weight="thin" size={18} />
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
