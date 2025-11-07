import { BroomIcon, PlayIcon, StopIcon } from "@phosphor-icons/react";
import { memo, useMemo } from "react";

import { IconButton } from "@flow/components";
import { useSubscription } from "@flow/lib/gql/subscriptions/useSubscription";
import { useT } from "@flow/lib/i18n";
import { useIndexedDB } from "@flow/lib/indexedDB";
import { useCurrentProject } from "@flow/stores";

import { DebugStartDialog, DebugStopDialog } from "./components";
import useHooks from "./hooks";

const tooltipOffset = 6;

type Props = {
  onDebugRunStart: () => Promise<void>;
  onDebugRunStop: () => Promise<void>;
};

const DebugActionBar: React.FC<Props> = ({
  onDebugRunStart,
  onDebugRunStop,
}) => {
  const t = useT();
  const {
    showDialog,
    debugRunStarted,
    jobStatus,
    debugJob,
    handleDebugRunStart,
    handleShowDebugStartDialog,
    handleShowDebugStopDialog,
    handleDialogClose,
    handleDebugRunReset,
  } = useHooks({ onDebugRunStart });

  return (
    <>
      <div className="flex items-center gap-2 align-middle">
        <StartButton
          debugRunStarted={debugRunStarted}
          onShowDialog={handleShowDebugStartDialog}
        />
        <IconButton
          className="shrink-0"
          tooltipText={t("Stop debug run of workflow")}
          tooltipOffset={tooltipOffset}
          disabled={
            !jobStatus || (jobStatus !== "running" && jobStatus !== "queued")
          }
          icon={<StopIcon weight="thin" size={18} />}
          onClick={handleShowDebugStopDialog}
        />
        <IconButton
          className="shrink-0"
          tooltipText={t("Clear debug run and results")}
          tooltipOffset={tooltipOffset}
          disabled={
            !debugJob ||
            !jobStatus ||
            jobStatus === "running" ||
            jobStatus === "queued"
          }
          icon={<BroomIcon weight="thin" size={18} />}
          onClick={handleDebugRunReset}
        />
      </div>
      {showDialog === "debugStart" && (
        <DebugStartDialog
          debugRunStarted={debugRunStarted}
          onDialogClose={handleDialogClose}
          onDebugRunStart={handleDebugRunStart}
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

export default memo(DebugActionBar);

const StartButton: React.FC<{
  debugRunStarted: boolean;
  onShowDialog: () => void;
}> = ({ debugRunStarted, onShowDialog }) => {
  const t = useT();
  const [currentProject] = useCurrentProject();

  const { value: debugRunState } = useIndexedDB("debugRun");

  const debugJobId = useMemo(
    () =>
      debugRunState?.jobs?.find((job) => job.projectId === currentProject?.id)
        ?.jobId,
    [debugRunState, currentProject],
  );

  const { data: jobStatus } = useSubscription(
    "GetSubscribedJobStatus",
    debugJobId,
    !debugJobId,
  );

  return (
    <IconButton
      className={`min-w-[36px] transition-all ${
        debugRunStarted || jobStatus
          ? `h-8 w-full rounded-lg bg-primary/50 px-4 ${jobStatus === "running" || jobStatus === "queued" ? "cursor-pointer" : ""}`
          : "w-[36px]"
      }`}
      disabled={
        debugRunStarted || jobStatus === "running" || jobStatus === "queued"
      }
      tooltipText={jobStatus ?? t("Start debug run of workflow")}
      tooltipOffset={tooltipOffset}
      delayDuration={200}
      icon={
        <div>
          {debugRunStarted || jobStatus ? (
            <div className="mr-1 flex items-center gap-2">
              <div
                className={`${
                  jobStatus === "completed"
                    ? "bg-success"
                    : jobStatus === "running"
                      ? "active-node-status"
                      : jobStatus === "cancelled"
                        ? "bg-warning"
                        : jobStatus === "failed"
                          ? "bg-destructive"
                          : jobStatus === "queued"
                            ? "queued-node-status"
                            : "bg-secondary"
                } size-3 rounded-full`}
              />
              <PlayIcon weight="thin" size={18} />
            </div>
          ) : (
            <PlayIcon weight="thin" size={18} />
          )}
        </div>
      }
      onClick={
        debugRunStarted || jobStatus === "running" || jobStatus === "queued"
          ? undefined
          : onShowDialog
      }
    />
  );
};
