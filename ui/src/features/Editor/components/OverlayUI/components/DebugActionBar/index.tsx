import {
  ArrowRightIcon,
  BroomIcon,
  CaretDownIcon,
  CircleIcon,
  PlayIcon,
  StopIcon,
} from "@phosphor-icons/react";
import { getConnectedEdges, useReactFlow } from "@xyflow/react";
import { memo, useMemo, useState } from "react";

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
import { useSubscription } from "@flow/lib/gql/subscriptions/useSubscription";
import { useT } from "@flow/lib/i18n";
import { useIndexedDB } from "@flow/lib/indexedDB";
import { JobState, useCurrentProject } from "@flow/stores";
import { AnyWorkflowVariable, AwarenessUser, Edge, Node } from "@flow/types";

import {
  DebugActiveRunsPopover,
  DebugStartPopover,
  DebugStopPopover,
  DebugWorkflowVariablesDialog,
} from "./components";
import useHooks from "./hooks";

const tooltipOffset = 6;

type Props = {
  activeUsersDebugRuns?: AwarenessUser[];
  selectedNodeIds: string[];
  edges?: Edge[];
  isSaving: boolean;
  onDebugRunJoin?: (jobId: string, userName: string) => Promise<void>;
  onDebugRunStart: () => Promise<void>;
  onDebugRunStartFromSelectedNode?: (
    node?: Node,
    nodes?: Node[],
  ) => Promise<void>;
  onDebugRunStop: () => Promise<void>;
  customDebugRunWorkflowVariables?: AnyWorkflowVariable[];
  onDebugRunVariableValueChange: (index: number, newValue: any) => void;
};

const DebugActionBar: React.FC<Props> = ({
  activeUsersDebugRuns,
  selectedNodeIds,
  edges,
  isSaving,
  customDebugRunWorkflowVariables,
  onDebugRunJoin,
  onDebugRunStart,
  onDebugRunStartFromSelectedNode,
  onDebugRunStop,
  onDebugRunVariableValueChange,
}) => {
  const t = useT();
  const {
    showOverlayElement,
    debugRunStarted,
    jobStatus,
    debugJob,
    handleDebugRunStart,
    handleDebugRunStop,
    handleShowDebugStartPopover,
    handleShowDebugStopPopover,
    handleShowDebugActiveRunsPopover,
    handleShowDebugWorkflowVariablesDialog,
    handlePopoverClose,
    handleDebugRunReset,
  } = useHooks({
    onDebugRunStart,
    onDebugRunStop,
    customDebugRunWorkflowVariables,
  });
  return (
    <div className="flex items-center gap-2 align-middle">
      <StartButton
        debugRunStarted={debugRunStarted}
        selectedNodeIds={selectedNodeIds}
        edges={edges}
        isSaving={isSaving}
        onShowDebugStartPopover={handleShowDebugStartPopover}
        onShowDebugWorkflowVariablesDialog={
          handleShowDebugWorkflowVariablesDialog
        }
        showPopover={showOverlayElement}
        onPopoverClose={handlePopoverClose}
        onDebugRunStart={handleDebugRunStart}
        onDebugRunStartFromSelectedNode={onDebugRunStartFromSelectedNode}
      />
      <StopButton
        jobStatus={jobStatus}
        onShowDebugStopPopover={handleShowDebugStopPopover}
        showPopover={showOverlayElement}
        onPopoverClose={handlePopoverClose}
        onDebugRunStop={handleDebugRunStop}
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
      <DebugActiveRunsPopover
        activeUsersDebugRuns={activeUsersDebugRuns}
        showPopover={showOverlayElement}
        onDebugRunJoin={onDebugRunJoin}
        onShowDebugRunsPopover={handleShowDebugActiveRunsPopover}
        onPopoverClose={handlePopoverClose}
        onDebugRunStart={onDebugRunStart}
      />
      {showOverlayElement === "debugWorkflowVariables" && (
        <DebugWorkflowVariablesDialog
          debugRunWorkflowVariables={customDebugRunWorkflowVariables}
          onDebugRunVariableValueChange={onDebugRunVariableValueChange}
          onDebugRunStart={onDebugRunStart}
          onDialogClose={handlePopoverClose}
        />
      )}
    </div>
  );
};

export default memo(DebugActionBar);

const StartButton: React.FC<{
  debugRunStarted: boolean;
  selectedNodeIds: string[];
  edges?: Edge[];
  isSaving: boolean;
  showPopover: string | undefined;
  onShowDebugStartPopover: () => void;
  onShowDebugWorkflowVariablesDialog: () => void;
  onDebugRunStart: () => Promise<void>;
  onDebugRunStartFromSelectedNode?: (
    node?: Node,
    nodes?: Node[],
  ) => Promise<void>;
  onPopoverClose: () => void;
}> = ({
  debugRunStarted,
  selectedNodeIds,
  edges,
  isSaving,
  showPopover,
  onDebugRunStart,
  onDebugRunStartFromSelectedNode,
  onShowDebugStartPopover,
  onPopoverClose,
}) => {
  const t = useT();
  const [currentProject] = useCurrentProject();

  const { value: debugRunState } = useIndexedDB("debugRun");

  const debugJob = useMemo(
    () =>
      debugRunState?.jobs?.find((job) => job.projectId === currentProject?.id),
    [debugRunState, currentProject],
  );

  const { data: jobStatus } = useSubscription(
    "GetSubscribedJobStatus",
    debugJob?.jobId,
    !debugJob,
  );

  return (
    <div>
      <Popover
        open={showPopover === "debugStart"}
        onOpenChange={(open) => {
          if (!open) onPopoverClose();
        }}>
        <PopoverTrigger asChild>
          <div className="group flex gap-1 rounded-md transition-all duration-300 ease-in-out">
            <IconButton
              className={`min-w-9 group-hover:bg-accent ${
                debugRunStarted || jobStatus
                  ? `h-8 w-full rounded-lg pr-1 pl-2 dark:bg-primary/50 ${jobStatus === "running" || jobStatus === "queued" ? "cursor-pointer" : ""}`
                  : "w-9"
              }`}
              disabled={
                isSaving ||
                debugRunStarted ||
                jobStatus === "running" ||
                jobStatus === "queued"
              }
              tooltipText={jobStatus ?? t("Start debug run of workflow")}
              tooltipOffset={tooltipOffset}
              delayDuration={200}
              icon={
                debugRunStarted || jobStatus ? (
                  <div className="flex items-center gap-2">
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
            <DebugRunDropDownMenu
              debugRunStarted={debugRunStarted}
              selectedNodeIds={selectedNodeIds}
              edges={edges}
              isSaving={isSaving}
              jobStatus={jobStatus}
              debugJob={debugJob}
              showPopover={showPopover}
              onShowDebugStartPopover={onShowDebugStartPopover}
              onDebugRunStartFromSelectedNode={onDebugRunStartFromSelectedNode}
            />
          </div>
        </PopoverTrigger>
        <PopoverContent
          sideOffset={8}
          collisionPadding={5}
          className="bg-primary/50 backdrop-blur">
          <DebugStartPopover
            debugRunStarted={debugRunStarted}
            onDebugRunStart={onDebugRunStart}
          />
        </PopoverContent>
      </Popover>
    </div>
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
          <DebugStopPopover onDebugRunStop={onDebugRunStop} />
        )}
      </PopoverContent>
    </Popover>
  );
};

const DebugRunDropDownMenu: React.FC<{
  debugRunStarted: boolean;
  selectedNodeIds: string[];
  edges?: Edge[];

  showPopover: string | undefined;
  isSaving: boolean;
  jobStatus: string | undefined;
  debugJob: JobState | undefined;
  onDebugRunStartFromSelectedNode?: (
    node?: Node,
    nodes?: Node[],
  ) => Promise<void>;

  onShowDebugStartPopover: () => void;
}> = ({
  debugRunStarted,
  selectedNodeIds,
  edges,
  isSaving,
  jobStatus,
  debugJob,
  onDebugRunStartFromSelectedNode,
  onShowDebugStartPopover,
}) => {
  const t = useT();
  const [showDropDownMenu, setShowDropDownMenu] = useState<boolean>(false);
  const { getNodes } = useReactFlow();
  const selectedNode =
    selectedNodeIds.length > 0
      ? (getNodes().find((node) => node.id === selectedNodeIds[0]) as
          | Node
          | undefined)
      : undefined;

  return (
    <DropdownMenu open={showDropDownMenu} onOpenChange={setShowDropDownMenu}>
      <DropdownMenuTrigger asChild>
        <IconButton
          className={`w-3 self-center rounded-sm group-hover:bg-accent ${showDropDownMenu ? "bg-accent" : ""}
          ${
            debugRunStarted || jobStatus
              ? `h-[32px] ${
                  jobStatus === "running" || jobStatus === "queued"
                    ? "cursor-pointer dark:bg-primary/50"
                    : ""
                }`
              : "h-[36px] w-3"
          }
        `}
          tooltipText={t("Additional Debug Actions")}
          tooltipOffset={tooltipOffset}
          icon={<CaretDownIcon size={18} weight="light" />}
        />
      </DropdownMenuTrigger>
      <DropdownMenuContent
        className="min-w-42.5 bg-primary/50 backdrop-blur select-none"
        align="start"
        sideOffset={8}
        alignOffset={-42}>
        <DropdownMenuItem
          className="flex items-center justify-between"
          disabled={debugRunStarted || isSaving}
          onClick={() => {
            setTimeout(() => {
              onShowDebugStartPopover();
            }, 180);
          }}>
          <div className="flex items-center gap-2">
            <PlayIcon weight="light" />
            <p>{t("Run Workflow")}</p>
          </div>
        </DropdownMenuItem>
        <DropdownMenuItem
          className="flex items-center justify-between"
          disabled={
            isSaving ||
            !selectedNode ||
            selectedNode.type === "batch" ||
            selectedNode.type === "note" ||
            selectedNode.type === "writer" ||
            selectedNode.type === "subworkflow" ||
            getConnectedEdges([selectedNode], edges ?? []).length === 0 ||
            selectedNodeIds.length > 1 ||
            !debugJob?.jobId ||
            debugJob.status !== "completed"
          }
          onClick={() => {
            setTimeout(() => {
              onDebugRunStartFromSelectedNode?.(selectedNode);
            }, 180);
          }}>
          <div className="flex items-center gap-2">
            <span className="relative flex items-center">
              <CircleIcon weight="fill" className="scale-60 transform" />
              <ArrowRightIcon
                weight="bold"
                className="absolute left-1.25 scale-80 transform"
              />
            </span>
            <p>{t("Run From Selected Action")}</p>
          </div>
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
};
