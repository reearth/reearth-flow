import { ColumnDef } from "@tanstack/react-table";
import { useCallback, useEffect, useMemo, useState } from "react";

import { LogsTable } from "@flow/components/LogsTable";
import { useJob } from "@flow/lib/gql/job";
import { useSubscription } from "@flow/lib/gql/subscriptions/useSubscription";
import { useT } from "@flow/lib/i18n";
import type { Log } from "@flow/types";
import { formatTimestamp } from "@flow/utils";
import { parseJSONL } from "@flow/utils/jsonl";

type LogsConsoleProps = {
  jobId: string;
};

const LogsConsole: React.FC<LogsConsoleProps> = ({ jobId }) => {
  const t = useT();
  const columns: ColumnDef<Log>[] = [
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
      accessorKey: "status",
      header: t("Status"),
    },
    {
      accessorKey: "message",
      header: t("Message"),
    },
  ];

  const [urlLogs, setUrlLogs] = useState<Log[] | null>(null);
  const [isFetchingLogsUrl, setIsFetchingLogsUrl] = useState<boolean>(false);

  const { useGetJob } = useJob();

  const debugJob = useGetJob(jobId).job;

  const { data: liveLogs } = useSubscription("GetSubscribedLogs", jobId);
  const { data: urlLogsData } = useSubscription(
    "GetSubscribedUserFacingLogs",
    jobId,
  );

  console.log("URL LOGS DATA", urlLogsData);

  const logs = useMemo(() => urlLogs || liveLogs || [], [liveLogs, urlLogs]);

  const getLogsFromUrl = useCallback(async () => {
    if (!debugJob || !debugJob.logsURL || debugJob.status !== "completed")
      return;
    setIsFetchingLogsUrl(true);
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
            nodeId: parsedLog.action,
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
    if (debugJob?.logsURL && !urlLogs) {
      (async () => {
        await getLogsFromUrl();
      })();
    }
  }, [debugJob?.logsURL, urlLogs, getLogsFromUrl]);

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
