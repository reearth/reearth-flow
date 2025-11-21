import { BroomIcon, PlayIcon, StopIcon } from "@phosphor-icons/react";
import { memo, useMemo } from "react";

import {
  IconButton,
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@flow/components";
import { useSubscription } from "@flow/lib/gql/subscriptions/useSubscription";
import { useT } from "@flow/lib/i18n";
import { useIndexedDB } from "@flow/lib/indexedDB";
import { useCurrentProject } from "@flow/stores";

import { DebugStartPopover, DebugStopPopover } from "./components";
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
    showPopover,
    debugRunStarted,
    jobStatus,
    debugJob,
    handleDebugRunStart,
    handleShowDebugStartPopover,
    handleShowDebugStopPopover,
    handlePopoverClose,
    handleDebugRunReset,
  } = useHooks({ onDebugRunStart });

  return (
    <div className="flex items-center gap-2 align-middle">
      <StartButton
        debugRunStarted={debugRunStarted}
        onShowDebugStartPopover={handleShowDebugStartPopover}
        showPopover={showPopover}
        onPopoverClose={handlePopoverClose}
        onDebugRunStart={handleDebugRunStart}
      />
      <StopButton
        jobStatus={jobStatus}
        onShowDebugStopPopover={handleShowDebugStopPopover}
        showPopover={showPopover}
        onPopoverClose={handlePopoverClose}
        onDebugRunStop={onDebugRunStop}
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
  );
};

export default memo(DebugActionBar);

const StartButton: React.FC<{
  debugRunStarted: boolean;
  showPopover: string | undefined;
  onShowDebugStartPopover: () => void;
  onDebugRunStart: () => Promise<void>;
  onPopoverClose: () => void;
}> = ({
  debugRunStarted,
  showPopover,
  onDebugRunStart,
  onShowDebugStartPopover,
  onPopoverClose,
}) => {
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
    <Popover
      open={showPopover === "debugStart"}
      onOpenChange={(open) => {
        if (!open) onPopoverClose();
      }}>
      <PopoverTrigger asChild>
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
            debugRunStarted || jobStatus ? (
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
            )
          }
          onClick={onShowDebugStartPopover}
        />
      </PopoverTrigger>
      <PopoverContent
        sideOffset={8}
        collisionPadding={5}
        className="bg-primary/50 backdrop-blur">
        {showPopover === "debugStart" && (
          <DebugStartPopover
            debugRunStarted={debugRunStarted}
            onPopoverClose={onPopoverClose}
            onDebugRunStart={onDebugRunStart}
          />
        )}
      </PopoverContent>
    </Popover>
  );
};

const StopButton: React.FC<{
  jobStatus: string | undefined;
  showPopover: string | undefined;
  onShowDebugStopPopover: () => void;
  onDebugRunStop: () => Promise<void>;
  onPopoverClose: () => void;
}> = ({
  jobStatus,
  showPopover,
  onDebugRunStop,
  onShowDebugStopPopover,
  onPopoverClose,
}) => {
  const t = useT();

  return (
    <Popover
      open={showPopover === "debugStop"}
      onOpenChange={(open) => {
        if (!open) onPopoverClose();
      }}>
      <PopoverTrigger asChild>
        <IconButton
          className="shrink-0"
          disabled={
            !jobStatus || (jobStatus !== "running" && jobStatus !== "queued")
          }
          tooltipText={t("Stop debug run of workflow")}
          tooltipOffset={tooltipOffset}
          icon={<StopIcon weight="thin" size={18} />}
          onClick={onShowDebugStopPopover}
        />
      </PopoverTrigger>
      <PopoverContent
        sideOffset={8}
        collisionPadding={5}
        className="bg-primary/50 backdrop-blur">
        {showPopover === "debugStop" && (
          <DebugStopPopover
            onPopoverClose={onPopoverClose}
            onDebugRunStop={onDebugRunStop}
          />
        )}
      </PopoverContent>
    </Popover>
  );
};
