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
      <div className="flex rounded-md border bg-secondary">
        <div className="flex align-middle">
          <IconButton
            className="rounded-l-[4px] rounded-r-none"
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
            className="rounded-none"
            tooltipText={t("Stop debug run of workflow")}
            tooltipOffset={tooltipOffset}
            disabled={
              !jobStatus || (jobStatus !== "running" && jobStatus !== "queued")
            }
            icon={<Stop weight="thin" />}
            onClick={handleShowDebugStopDialog}
          />
          <IconButton
            className="rounded-none"
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
            className="rounded-none"
            tooltipText={t("Deploy project's workflow")}
            tooltipOffset={tooltipOffset}
            icon={<RocketLaunch weight="thin" />}
            onClick={handleShowDeployDialog}
          />
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <IconButton
                className="w-[25px] rounded-l-none rounded-r-[4px]"
                tooltipText={t("Additional actions")}
                tooltipOffset={tooltipOffset}
                icon={<DotsThreeVertical />}
              />
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
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
