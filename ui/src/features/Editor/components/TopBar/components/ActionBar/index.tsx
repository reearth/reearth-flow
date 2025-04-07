import {
  Broom,
  DotsThreeVertical,
  Export,
  LetterCircleV,
  Play,
  RocketLaunch,
  ShareFat,
  Stop,
} from "@phosphor-icons/react";
import { memo } from "react";

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  IconButton,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";

import { DebugStopDialog, DeployDialog, ShareDialog } from "./components";
import useHooks from "./hooks";

const tooltipOffset = 6;

type Props = {
  allowedToDeploy: boolean;
  onWorkflowDeployment: (
    description: string,
    deploymentId?: string,
  ) => Promise<void>;
  onProjectShare: (share: boolean) => void;
  onDebugRunStart: () => Promise<void>;
  onDebugRunStop: () => Promise<void>;
  onRightPanelOpen: (content?: "version-history") => void;
};

const ActionBar: React.FC<Props> = ({
  allowedToDeploy,
  onWorkflowDeployment,
  onProjectShare,
  onDebugRunStart,
  onDebugRunStop,
  onRightPanelOpen,
}) => {
  const t = useT();
  const {
    showDialog,
    debugRunStarted,
    jobStatus,
    debugJob,
    handleDebugRunStart,
    handleShowDeployDialog,
    handleShowShareDialog,
    handleShowDebugStopDialog,
    handleDialogClose,
    handleDebugRunReset,
    handleProjectExport,
  } = useHooks({ onDebugRunStart });

  return (
    <>
      <div className="flex rounded-md bg-secondary">
        <div className="flex align-middle gap-1">
          <IconButton
            className="rounded-l-[4px] rounded"
            tooltipText={t("Start debug run of workflow")}
            tooltipOffset={tooltipOffset}
            disabled={
              debugRunStarted ||
              jobStatus === "running" ||
              jobStatus === "queued"
            }
            icon={<Play weight="thin" />}
            onClick={handleDebugRunStart}
          />
          <IconButton
            tooltipText={t("Stop debug run of workflow")}
            tooltipOffset={tooltipOffset}
            disabled={
              !jobStatus || (jobStatus !== "running" && jobStatus !== "queued")
            }
            icon={<Stop weight="thin" />}
            onClick={handleShowDebugStopDialog}
          />
          <IconButton
            tooltipText={t("Clear debug run and results")}
            tooltipOffset={tooltipOffset}
            disabled={
              !debugJob ||
              !jobStatus ||
              jobStatus === "running" ||
              jobStatus === "queued"
            }
            icon={<Broom weight="thin" />}
            onClick={handleDebugRunReset}
          />
          <IconButton
            tooltipText={t("Deploy project's workflow")}
            tooltipOffset={tooltipOffset}
            icon={<RocketLaunch weight="thin" />}
            onClick={handleShowDeployDialog}
          />
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <IconButton
                className="w-[25px]"
                tooltipText={t("Additional actions")}
                tooltipOffset={tooltipOffset}
                icon={<DotsThreeVertical />}
              />
            </DropdownMenuTrigger>
            <DropdownMenuContent
              className="flex flex-col gap-2"
              align="end"
              sideOffset={10}
              alignOffset={2}>
              <DropdownMenuItem
                className="flex justify-between gap-4"
                onClick={handleShowShareDialog}>
                <p>{t("Share Project")}</p>
                <ShareFat weight="light" />
              </DropdownMenuItem>
              <DropdownMenuItem
                className="flex justify-between gap-4"
                onClick={handleProjectExport}>
                <p>{t("Export Project")}</p>
                <Export weight="light" />
              </DropdownMenuItem>
              <DropdownMenuItem
                className="flex justify-between gap-4"
                onClick={() => onRightPanelOpen("version-history")}>
                <p>{t("Version History")}</p>
                <LetterCircleV weight="light" />
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </div>
      {showDialog === "deploy" && (
        <DeployDialog
          allowedToDeploy={allowedToDeploy}
          onDialogClose={handleDialogClose}
          onWorkflowDeployment={onWorkflowDeployment}
        />
      )}
      {showDialog === "share" && (
        <ShareDialog
          onDialogClose={handleDialogClose}
          onProjectShare={onProjectShare}
        />
      )}
      {showDialog === "debugStop" && (
        <DebugStopDialog
          onDialogClose={handleDialogClose}
          onDebugRunStop={onDebugRunStop}
        />
      )}
    </>
  );
};

export default memo(ActionBar);
