import {
  DotsThreeVertical,
  Export,
  LetterCircleV,
  Play,
  RocketLaunch,
  ShareFat,
  Stop,
  XCircle,
} from "@phosphor-icons/react";
import { memo, useMemo, useState } from "react";

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  IconButton,
} from "@flow/components";
import { useProjectExport } from "@flow/hooks";
import { useJob } from "@flow/lib/gql/job";
import { useT } from "@flow/lib/i18n";
import { useIndexedDB } from "@flow/lib/indexedDB";
import { useCurrentProject } from "@flow/stores";

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

  const { useGetJob } = useJob();

  const { value: debugRunState, updateValue: updateDebugRunState } =
    useIndexedDB("debugRun");

  const debugJobId = useMemo(
    () =>
      debugRunState?.jobs?.find((job) => job.projectId === currentProject?.id)
        ?.jobId ?? "",
    [debugRunState, currentProject],
  );

  const debugJob = useGetJob(debugJobId).job;

  const [showDialog, setShowDialog] = useState<
    "deploy" | "share" | "debugStop" | undefined
  >(undefined);

  const handleShowDeployDialog = () => setShowDialog("deploy");
  const handleShowShareDialog = () => setShowDialog("share");
  const handleShowDebugStopDialog = () => setShowDialog("debugStop");
  const handleDialogClose = () => setShowDialog(undefined);

  const handleDebugRunReset = async () => {
    const jobState = debugRunState?.jobs?.filter(
      (job) => job.projectId !== currentProject?.id,
    );
    if (!jobState) return;
    await updateDebugRunState({ jobs: jobState });
  };

  return (
    <>
      <div className="flex rounded-md border bg-secondary">
        <div className="flex align-middle">
          <IconButton
            className="rounded-l-[4px] rounded-r-none"
            tooltipText={t("Run project workflow")}
            tooltipOffset={tooltipOffset}
            disabled={
              debugJob &&
              (debugJob.status === "running" || debugJob.status === "queued")
            }
            icon={<Play weight="thin" />}
            onClick={onDebugRunStart}
          />
          <IconButton
            className="rounded-none"
            tooltipText={t("Stop project workflow")}
            tooltipOffset={tooltipOffset}
            disabled={
              !debugJob ||
              (debugJob &&
                !(
                  debugJob.status === "running" || debugJob.status === "queued"
                ))
            }
            icon={<Stop weight="thin" />}
            onClick={handleShowDebugStopDialog}
          />
          <IconButton
            className="rounded-none"
            tooltipText={t("Clear debug run results")}
            tooltipOffset={tooltipOffset}
            disabled={!debugJob?.outputURLs}
            icon={<XCircle weight="thin" />}
            onClick={handleDebugRunReset}
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
