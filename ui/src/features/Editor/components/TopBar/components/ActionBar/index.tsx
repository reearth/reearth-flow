import {
  ClockCounterClockwiseIcon,
  DotsThreeVerticalIcon,
  ExportIcon,
  PaperPlaneTiltIcon,
  RocketIcon,
  FloppyDiskIcon,
} from "@phosphor-icons/react";
import { memo } from "react";
import { Doc } from "yjs";

import {
  ContextMenuShortcut,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  IconButton,
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { Project } from "@flow/types";

import { DeployDialog, SharePopover } from "./components";
import { VersionDialog } from "./components/Version/VersionDialog";
import useHooks from "./hooks";

const tooltipOffset = 6;

type Props = {
  project?: Project;
  yDoc: Doc | null;
  allowedToDeploy: boolean;
  onWorkflowDeployment: (
    description: string,
    deploymentId?: string,
  ) => Promise<void>;
  onProjectShare: (share: boolean) => void;
  onProjectExport: () => void;
};

const ActionBar: React.FC<Props> = ({
  project,
  yDoc,
  allowedToDeploy,
  onWorkflowDeployment,
  onProjectShare,
  onProjectExport,
}) => {
  const t = useT();
  const {
    showDialog,
    isSaving,
    handleShowDeployDialog,
    handleShowVersionDialog,
    handleShowSharePopover,
    handleDialogClose,
    handleProjectSnapshotSave,
  } = useHooks({ projectId: project?.id ?? "" });

  return (
    <>
      <div className="flex rounded-md bg-secondary">
        <div className="flex gap-2 align-middle">
          <IconButton
            tooltipText={t("Deploy project's workflow")}
            tooltipOffset={tooltipOffset}
            icon={<RocketIcon weight="thin" size={18} />}
            onClick={handleShowDeployDialog}
          />
          <Popover
            open={showDialog === "share"}
            onOpenChange={(open) => {
              if (!open) handleDialogClose();
            }}>
            <PopoverTrigger asChild>
              <IconButton
                tooltipText={t("Share Project")}
                tooltipOffset={tooltipOffset}
                icon={<PaperPlaneTiltIcon weight="thin" size={18} />}
                onClick={handleShowSharePopover}
              />
            </PopoverTrigger>
            <PopoverContent>
              {showDialog === "share" && (
                <SharePopover onProjectShare={onProjectShare} />
              )}
            </PopoverContent>
          </Popover>
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <IconButton
                className="w-[25px]"
                tooltipText={t("Additional actions")}
                tooltipOffset={tooltipOffset}
                icon={<DotsThreeVerticalIcon size={18} />}
              />
            </DropdownMenuTrigger>
            <DropdownMenuContent
              className="min-w-[170px] rounded-md bg-primary p-1 text-popover-foreground shadow-md select-none"
              align="end"
              sideOffset={10}
              alignOffset={2}>
              <DropdownMenuItem
                className="flex items-center justify-between rounded-sm px-2 py-1.5 text-xs"
                onSelect={(e) => {
                  e.preventDefault();
                }}
                disabled={isSaving}
                onClick={handleProjectSnapshotSave}>
                <div className="flex items-center gap-1">
                  <FloppyDiskIcon weight="light" />
                  <p>{t("Manual Save")}</p>
                </div>

                <div className="flex flex-row gap-1">
                  <ContextMenuShortcut
                    keyBinding={{ key: "s", commandKey: true }}
                  />
                </div>
              </DropdownMenuItem>
              <DropdownMenuItem
                className="flex items-center justify-between rounded-sm px-2 py-1.5 text-xs"
                onClick={handleShowVersionDialog}>
                <div className="flex items-center gap-1">
                  <ClockCounterClockwiseIcon weight="light" />
                  <p>{t("Version History")}</p>
                </div>
              </DropdownMenuItem>
              <DropdownMenuItem
                className="flex items-center justify-between rounded-sm px-2 py-1.5 text-xs"
                onClick={onProjectExport}>
                <div className="flex items-center gap-1">
                  <ExportIcon weight="light" />
                  <p>{t("Export Project")}</p>
                </div>
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </div>
      {showDialog === "version" && (
        <VersionDialog
          project={project}
          yDoc={yDoc}
          onDialogClose={handleDialogClose}
        />
      )}
      {showDialog === "deploy" && (
        <DeployDialog
          allowedToDeploy={allowedToDeploy}
          onDialogClose={handleDialogClose}
          onWorkflowDeployment={onWorkflowDeployment}
        />
      )}
    </>
  );
};

export default memo(ActionBar);
