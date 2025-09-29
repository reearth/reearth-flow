import { DotsThreeVerticalIcon, ExportIcon } from "@phosphor-icons/react";
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
    <div className="flex h-[42px] w-[100vw] shrink-0 justify-between bg-secondary">
      <div className="flex items-center gap-4 border-b pr-2 pl-4">
        <div onClick={() => navigate({ to: "/" })}>
          <FlowLogo className="size-7 cursor-pointer transition-all" />
        </div>
      </div>
      <div className="flex items-center border-b pr-4 pl-2">
        <p className="flex max-w-[500px] items-center gap-2 truncate text-sm transition-all delay-0 duration-500 dark:font-thin">
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
      <div className="flex items-center justify-center gap-1 border-b pr-1">
        <Button
          type="button"
          variant="outline"
          size="sm"
          disabled={!me}
          onClick={handleShowImportDialog}>
          {t("Import into Workspace")}
        </Button>
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
