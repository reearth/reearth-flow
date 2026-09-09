import { memo, useEffect, useState } from "react";

import DiagnosticsConsole from "@flow/features/DiagnosticsConsole";
import LogsConsole from "@flow/features/LogsConsole";
import useJobDiagnostics from "@flow/hooks/useJobDiagnostics";

import ViewSwitch, { type DebugLogsView } from "./ViewSwitch";

type Props = {
  debugJobId?: string;
  isJobActive?: boolean;
};

/**
 * The run's output: the log stream and the structured diagnostics, under one
 * tab with a switch between them rather than two tabs to choose from.
 */
const DebugLogs: React.FC<Props> = ({ debugJobId, isJobActive }) => {
  const [view, setView] = useState<DebugLogsView>("logs");

  // Reads the same query keys the diagnostics view reads, so the badge can
  // never disagree with what the table shows, and it costs no extra request.
  const { count, hasBlocking } = useJobDiagnostics(debugJobId, isJobActive);

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
          leadingActions={switchControl}
        />
      ) : (
        <LogsConsole jobId={debugJobId} leadingActions={switchControl} />
      )}
    </div>
  );
};

export default memo(DebugLogs);
