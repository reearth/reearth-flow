import { ArrowSquareIn, Export } from "@phosphor-icons/react";
import { memo } from "react";
import { Doc } from "yjs";

import { IconButton } from "@flow/components";
import { WorkflowTabs } from "@flow/features/Editor/components";
import { useT } from "@flow/lib/i18n";
import { Me, Project, Workspace } from "@flow/types";

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
    handleProjectExport,
    handleProjectImport,
    handleDialogClose,
    handleShowImportDialog,
  } = useHooks({ yDoc, project, accessToken });
  const t = useT();
  return (
    <div className="flex shrink-0 justify-between gap-2 bg-secondary h-[44px] w-[100vw]">
      <div className="flex flex-1 gap-2 h-full overflow-hidden">
        <WorkflowTabs
          currentWorkflowId={currentWorkflowId}
          openWorkflows={openWorkflows}
          onWorkflowClose={onWorkflowClose}
          onWorkflowChange={onWorkflowChange}
        />
      </div>
      <div className="flex items-center">
        <IconButton
          tooltipText={t("Export Project")}
          tooltipOffset={6}
          icon={<Export weight="thin" size={18} />}
          onClick={handleProjectExport}
        />
        <IconButton
          tooltipText={t("Import Project")}
          tooltipOffset={6}
          icon={<ArrowSquareIn weight="thin" size={18} />}
          disabled={!me}
          onClick={handleShowImportDialog}
        />
        {showDialog === "import" && workspaces && (
          <ImportDialog
            workspaces={workspaces}
            selectedWorkspace={selectedWorkspace}
            onSelectWorkspace={handleSelectWorkspace}
            onImportProject={handleProjectImport}
            onDialogClose={handleDialogClose}
          />
        )}
      </div>
    </div>
  );
};

export default memo(SharedCanvasTopBar);
