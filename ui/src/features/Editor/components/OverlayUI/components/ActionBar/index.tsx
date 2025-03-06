import {
  DotsThreeVertical,
  Export,
  LetterCircleV,
  Play,
  RocketLaunch,
  ShareFat,
  Stop,
} from "@phosphor-icons/react";
import { memo, useState } from "react";

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  IconButton,
} from "@flow/components";
import { useProjectExport } from "@flow/hooks";
import { useT } from "@flow/lib/i18n";
import { loadStateFromIndexedDB, useCurrentProject } from "@flow/stores";

import { DebugStopDialog, DeployDialog, ShareDialog } from "./components";

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
  const [currentProject] = useCurrentProject();

  const { handleProjectExport } = useProjectExport(currentProject);

  const [showDeployDialog, setShowDeployDialog] = useState(false);
  const [showShareDialog, setShowShareDialog] = useState(false);
  const [showDebugStopDialog, setShowDebugStopDialog] = useState(false);

  const [debugWorkflowRunning, setDebugWorkflowRunning] = useState(false);

  const handleShowDeployDialog = () => {
    setShowShareDialog(false);
    setShowDebugStopDialog(false);
    setShowDeployDialog(true);
  };

  const handleShowShareDialog = () => {
    setShowDeployDialog(false);
    setShowDebugStopDialog(false);
    setShowShareDialog(true);
  };

  const handleShowDebugStopDialog = () => {
    setShowShareDialog(false);
    setShowDeployDialog(false);
    setShowDebugStopDialog(true);
  };

  const handleDebugRun = async (callback: () => Promise<void>) => {
    await callback();
    const debugRunState = await loadStateFromIndexedDB("debugRun");
    const debugJob = debugRunState?.jobs?.find(
      (job) => job.projectId === currentProject?.id,
    );
    if (debugJob && !debugWorkflowRunning) {
      setDebugWorkflowRunning(true);
    } else if (!debugJob && debugWorkflowRunning) {
      setDebugWorkflowRunning(false);
    }
  };

  const handleDebugRunStart = async () => {
    await handleDebugRun(onDebugRunStart);
  };

  const handleDebugRunStop = async () => {
    await handleDebugRun(onDebugRunStop);
  };

  return (
    <>
      <div className="rounded-md border bg-secondary">
        <div className="flex rounded-md">
          <div className="flex align-middle">
            <IconButton
              className="rounded-l-[4px] rounded-r-none"
              tooltipText={t("Run project workflow")}
              tooltipOffset={tooltipOffset}
              disabled={debugWorkflowRunning}
              icon={<Play weight="thin" />}
              onClick={handleDebugRunStart}
            />
            <IconButton
              className="rounded-none"
              tooltipText={t("Stop project workflow")}
              tooltipOffset={tooltipOffset}
              disabled={!debugWorkflowRunning}
              icon={<Stop weight="thin" />}
              onClick={handleShowDebugStopDialog}
            />
            <IconButton
              className="rounded-none"
              tooltipText={t("Deploy project workflow")}
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
      </div>
      {showDeployDialog && (
        <DeployDialog
          allowedToDeploy={allowedToDeploy}
          setShowDialog={setShowDeployDialog}
          onWorkflowDeployment={onWorkflowDeployment}
        />
      )}
      {showShareDialog && (
        <ShareDialog
          setShowDialog={setShowShareDialog}
          onProjectShare={onProjectShare}
        />
      )}
      {showDebugStopDialog && (
        <DebugStopDialog
          onDebugRunStop={handleDebugRunStop}
          setShowDialog={setShowDebugStopDialog}
        />
      )}
    </>
  );
};

export default memo(ActionBar);
