import { CaretUp, CaretDown, Minus, Terminal } from "@phosphor-icons/react";
import {
  memo,
  MouseEvent,
  useCallback,
  useEffect,
  useMemo,
  useState,
} from "react";

import { LoadingSkeleton } from "@flow/components";
import LogsConsole from "@flow/features/LogsConsole";
import { useJob } from "@flow/lib/gql/job";
import { useT } from "@flow/lib/i18n";
import { useIndexedDB } from "@flow/lib/indexedDB";
import mockLogs from "@flow/mock_data/logsv2Data";
import { useCurrentProject } from "@flow/stores";
import { Log } from "@flow/types";
import { parseJSONL } from "@flow/utils/parseJsonL";

const DebugLogs: React.FC = () => {
  const t = useT();
  const [expanded, setExpanded] = useState(false);
  const [minimized, setMinimized] = useState(false);
  const [showLogs, setShowLogs] = useState(false);

  const [currentProject] = useCurrentProject();

  const { useGetJob } = useJob();

  const { value: debugRunState } = useIndexedDB("debugRun");

  const debugJobId = useMemo(
    () =>
      debugRunState?.jobs?.find((job) => job.projectId === currentProject?.id)
        ?.jobId ?? "",
    [debugRunState, currentProject],
  );

  const debugJob = useGetJob(debugJobId).job;

  const [logs, setLogs] = useState<Log[] | undefined>(undefined);
  const [isFetching, setIsFetching] = useState<boolean>(false);

  const getAllLogs = useCallback(async () => {
    if (!debugJob || !debugJob.logsURL) return;
    setIsFetching(true);
    try {
      const response = await fetch(debugJob.logsURL);
      const textData = await response.text();

      // Logs are JSONL there we have ensure they are parsed correctly and cleaned to be used
      const logsArray = parseJSONL(textData, {
        transform: (parsedLog) => {
          if (
            typeof parsedLog.msg === "string" &&
            parsedLog.msg.trim() !== ""
          ) {
            try {
              parsedLog.msg = JSON.parse(parsedLog.msg);
            } catch (innerError) {
              console.error("Failed to clean msg:", parsedLog.msg, innerError);
            }
          }
          return {
            workflowId: debugJob.workspaceId,
            jobId: debugJob.id,
            message: parsedLog.msg,
            timestamp: parsedLog.ts,
            status: parsedLog.level,
          };
        },
        onError: (error, line, index) => {
          console.warn(
            `Skipping malformed log at line ${index}:`,
            line.substring(0, 100),
          );
          console.log("Error:", error);
        },
      });
      setLogs(logsArray);
    } catch (error) {
      console.error("Error fetching logs:", error);
    } finally {
      setIsFetching(false);
    }
  }, [debugJob, setIsFetching]);

  useEffect(() => {
    if (!logs) {
      getAllLogs();
    }
  }, [logs, getAllLogs]);

  console.log("value", logs);

  useEffect(() => {
    const debugJob = debugRunState?.jobs?.find(
      (job) => job.projectId === currentProject?.id,
    );
    if (debugJob) {
      setShowLogs(true);
    } else {
      setShowLogs(false);
    }
  }, [debugRunState, currentProject]);

  const handleExpand = () => {
    if (minimized) {
      setMinimized(false);
    } else {
      setExpanded((prev) => !prev);
    }
  };

  const handleMinimize = (e: MouseEvent) => {
    e.stopPropagation();
    setMinimized((prev) => !prev);
  };

  return showLogs ? (
    <div
      className={`pointer-events-auto w-[45vw] min-w-[700px] cursor-pointer rounded border bg-secondary transition-all ${minimized ? "h-[36px]" : expanded ? "h-full" : "h-[300px]"}`}>
      <div className="flex items-center px-1 pt-1" onClick={handleExpand}>
        <div className="flex flex-1 items-center justify-center gap-2">
          <Terminal />
          <p className="text-sm">{t("Workflow Logs")}</p>
        </div>
        <div className="flex items-center gap-2">
          <div
            className="rounded p-1 hover:bg-primary"
            onClick={handleMinimize}>
            <Minus />
          </div>
          <div className="rounded p-1 hover:bg-primary">
            {expanded && !minimized ? <CaretDown /> : <CaretUp />}
          </div>
        </div>
      </div>
      <div className="h-[calc(100%-24px)] overflow-scroll pt-1">
        {isFetching ? (
          <LoadingSkeleton />
        ) : (logs ?? mockLogs) ? (
          <LogsConsole data={logs || (mockLogs as Log[])} />
        ) : null}
      </div>
    </div>
  ) : null;
};

export default memo(DebugLogs);
