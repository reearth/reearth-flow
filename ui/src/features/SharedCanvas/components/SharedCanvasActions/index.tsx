import { DotsThreeVerticalIcon, ExportIcon } from "@phosphor-icons/react";
import { memo } from "react";
import { Doc } from "yjs";

import {
  Button,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  IconButton,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import type { Me, Project, Workspace } from "@flow/types";

import ImportDialog from "../ImportDialog";

import useHooks from "./hooks";

type Props = {
  yDoc: Doc | null;
  project?: Project;
  accessToken?: string;
  isMainWorkflow: boolean;
  me?: Me;
  workspaces?: Workspace[] | undefined;
};

const SharedCanvasActions: React.FC<Props> = ({
  yDoc,
  project,
  isMainWorkflow,
  accessToken,
  me,
  workspaces,
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
    <div
      className={`flex items-center justify-center gap-1 pr-1 ${isMainWorkflow ? "border-transparent" : "border-node-subworkflow"}`}>
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
            variant="ghost"
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
            disabled
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
  );
};

export default memo(SharedCanvasActions);
