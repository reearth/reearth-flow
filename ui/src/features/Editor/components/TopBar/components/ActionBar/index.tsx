import {
  ClockCounterClockwise,
  DotsThreeVertical,
  Export,
  PaperPlaneTilt,
  Rocket,
} from "@phosphor-icons/react";
import { memo } from "react";
import { Doc } from "yjs";

import {
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
    handleShowDeployDialog,
    handleShowVersionDialog,
    handleShowSharePopover,
    handleDialogClose,
  } = useHooks();

  return (
    <>
      <div className="flex rounded-md bg-secondary">
        <div className="flex align-middle gap-2">
          <IconButton
            tooltipText={t("Deploy project's workflow")}
            tooltipOffset={tooltipOffset}
            icon={<Rocket weight="thin" size={18} />}
            onClick={handleShowDeployDialog}
          />
          <Popover
            open={showDialog === "share"}
            onOpenChange={(open) => {
              if (!open) handleDialogClose();
            }}>
            <PopoverTrigger>
              <IconButton
                tooltipText={t("Share Project")}
                tooltipOffset={tooltipOffset}
                icon={<PaperPlaneTilt weight="thin" size={18} />}
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
                icon={<DotsThreeVertical size={18} />}
              />
            </DropdownMenuTrigger>
            <DropdownMenuContent
              className="flex flex-col gap-2"
              align="end"
              sideOffset={10}
              alignOffset={2}>
              <DropdownMenuItem
                className="flex justify-between gap-4"
                onClick={onProjectExport}>
                <p>{t("Export Project")}</p>
                <Export weight="thin" size={18} />
              </DropdownMenuItem>
              <DropdownMenuItem
                className="flex justify-between gap-4"
                onClick={handleShowVersionDialog}>
                <p>{t("Version History")}</p>
                <ClockCounterClockwise weight="thin" size={18} />
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
