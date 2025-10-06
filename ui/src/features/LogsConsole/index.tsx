import { ColumnDef } from "@tanstack/react-table";
import { useCallback, useEffect, useMemo, useState } from "react";

import { LogsTable } from "@flow/components/LogsTable";
import { useJob } from "@flow/lib/gql/job";
import { useSubscription } from "@flow/lib/gql/subscriptions/useSubscription";
import { useT } from "@flow/lib/i18n";
import type { UserFacingLog } from "@flow/types";
import { formatTimestamp } from "@flow/utils";
import { parseJSONL } from "@flow/utils/jsonl";

type LogsConsoleProps = {
  jobId: string;
};

const LogsConsole: React.FC<LogsConsoleProps> = ({ jobId }) => {
  const t = useT();
  const columns: ColumnDef<UserFacingLog>[] = [
    {
      accessorKey: "timestamp",
      header: t("Timestamp"),
      cell: ({ getValue }) => formatTimestamp(getValue<string>()),
    },
    {
      accessorKey: "nodeId",
      header: t("Node Id"),
    },
    {
      accessorKey: "nodeName",
      header: t("Node Name"),
    },
    {
      accessorKey: "level",
      header: t("Status"),
    },
    {
      accessorKey: "message",
      header: t("Message"),
    },
  ];

  const [urlLogs, setUrlLogs] = useState<UserFacingLog[] | null>(null);
  const [isFetchingLogsUrl, setIsFetchingLogsUrl] = useState<boolean>(false);

  const { useGetJob } = useJob();

  const debugJob = useGetJob(jobId).job;

  const { data: liveLogs } = useSubscription(
    "GetSubscribedUserFacingLogs",
    jobId,
  );

  const logs = useMemo(() => urlLogs || liveLogs || [], [liveLogs, urlLogs]);

  const getLogsFromUrl = useCallback(async () => {
    if (
      !debugJob ||
      !debugJob.userFacingLogsURL ||
      debugJob.status !== "completed"
    )
      return;
    setIsFetchingLogsUrl(true);
    try {
      const response = await fetch(debugJob.userFacingLogsURL);
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
            nodeId: parsedLog.nodeId,
            jobId: debugJob.id,
            message: parsedLog.message,
            timestamp: parsedLog.timestamp,
            level: parsedLog.level,
            nodeName: parsedLog.nodeName,
          };
        },
        onError: (error, line, index) => {
          console.warn(
            `Skipping malformed log at line ${index}:`,
            line.substring(0, 100),
          );
          console.error("Error:", error);
        },
      });
      setUrlLogs(logsArray);
    } catch (error) {
      console.error("Error fetching logs:", error);
    } finally {
      setIsFetchingLogsUrl(false);
    }
  }, [debugJob, setIsFetchingLogsUrl]);

  useEffect(() => {
    if (debugJob?.userFacingLogsURL && !urlLogs) {
      (async () => {
        await getLogsFromUrl();
      })();
    }
  }, [debugJob?.userFacingLogsURL, urlLogs, getLogsFromUrl]);

  return (
    <LogsTable
      columns={columns}
      data={logs}
      isFetching={!logs.length || isFetchingLogsUrl}
      selectColumns
      showFiltering
    />
  );
};

export default LogsConsole;
