import { memo, useCallback, useEffect, useState } from "react";

import DiagnosticsConsole from "@flow/features/DiagnosticsConsole";
import { useEditorContext } from "@flow/features/Editor/editorContext";
import LogsConsole from "@flow/features/LogsConsole";
import useJobDiagnostics from "@flow/hooks/useJobDiagnostics";

import ViewSwitch, { type DebugLogsView } from "./ViewSwitch";

type Props = {
  debugJobId?: string;
  isJobActive?: boolean;
  /** Drops the panel out of fullscreen, which otherwise hides the canvas. */
  onExitFullscreen?: () => void;
};

/**
 * The run's output: the log stream and the structured diagnostics, under one
 * tab with a switch between them rather than two tabs to choose from.
 */
const DebugLogs: React.FC<Props> = ({
  debugJobId,
  isJobActive,
  onExitFullscreen,
}) => {
  const [view, setView] = useState<DebugLogsView>("logs");

  // Diagnostics are bucketed per node with no job-wide query, so the ids are
  // what make them reachable — see `nodeDiagnosticsBatch`.
  const { workflowNodeIds, onNodeNavigate } = useEditorContext();

  // Revealing an action is pointless while the panel covers the whole screen,
  // so the fly-to only happens once the canvas is back in view.
  const handleNodeNavigate = useCallback(
    (nodeId: string) => {
      onExitFullscreen?.();
      onNodeNavigate?.(nodeId);
    },
    [onExitFullscreen, onNodeNavigate],
  );

  // Reads the same query keys the diagnostics view reads, so the badge can
  // never disagree with what the table shows, and it costs no extra request.
  const { count, hasBlocking } = useJobDiagnostics(
    debugJobId,
    isJobActive,
    workflowNodeIds,
  );

  // A new run, or one whose diagnostics aged out of the live cache, must not
  // leave the panel stranded on an inert view.
  useEffect(() => {
    if (!count) setView("logs");
  }, [count]);

  if (!debugJobId) return null;

  const switchControl = (
    <ViewSwitch
      view={view}
      diagnosticsCount={count}
      hasBlocking={hasBlocking}
      onViewChange={setView}
    />
  );

  return (
    <div className="h-[calc(100%-32px)] overflow-hidden pt-1">
      {view === "diagnostics" ? (
        <DiagnosticsConsole
          jobId={debugJobId}
          isJobActive={isJobActive}
          nodeIds={workflowNodeIds}
          leadingActions={switchControl}
          onNodeNavigate={onNodeNavigate ? handleNodeNavigate : undefined}
        />
      ) : (
        <LogsConsole jobId={debugJobId} leadingActions={switchControl} />
      )}
    </div>
  );
};

export default memo(DebugLogs);
