import { useEffect, useMemo, useState } from "react";
import type { ReactNode } from "react";

import { LogsTable } from "@flow/components/LogsTable";
import { useJob } from "@flow/lib/gql/job";
import { useSubscription } from "@flow/lib/gql/subscriptions/useSubscription";
import { useT } from "@flow/lib/i18n";
import type { AppColumnDef } from "@flow/lib/table/features";
import type { UserFacingLog } from "@flow/types";
import { formatTimestamp } from "@flow/utils";
import { parseJSONL } from "@flow/utils/jsonl";

type LogsConsoleProps = {
  jobId: string;
  leadingActions?: ReactNode;
};

const LogsConsole: React.FC<LogsConsoleProps> = ({ jobId, leadingActions }) => {
  const t = useT();
  const columns: AppColumnDef<UserFacingLog>[] = [
    {
      accessorKey: "timestamp",
      header: t("Timestamp"),
      cell: ({ getValue }) => formatTimestamp(getValue<string>()),
    },
    {
      accessorKey: "nodeId",
      header: t("Action Id"),
    },
    {
      accessorKey: "nodeName",
      header: t("Action Name"),
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

  const { useGetJob } = useJob();

  const debugJob = useGetJob(jobId).job;

  const { data: liveLogs } = useSubscription(
    "GetSubscribedUserFacingLogs",
    jobId,
  );

  const [fetchedLogs, setFetchedLogs] = useState<{
    jobId: string;
    logs: UserFacingLog[];
  } | null>(null);
  const [failedJobId, setFailedJobId] = useState<string | null>(null);

  const logsUrl =
    debugJob?.id === jobId && debugJob.status === "completed"
      ? debugJob.userFacingLogsURL
      : undefined;

  const urlLogs = fetchedLogs?.jobId === jobId ? fetchedLogs.logs : null;
  const hasFetched = urlLogs !== null;

  const isFetchingLogsUrl = !!logsUrl && !hasFetched && failedJobId !== jobId;

  const logs = useMemo(() => urlLogs || liveLogs || [], [liveLogs, urlLogs]);

  useEffect(() => {
    if (!logsUrl || hasFetched) return;

    // Aborted on cleanup, so a response for a job this console has moved on
    // from is dropped instead of landing on the next one.
    const controller = new AbortController();
    (async () => {
      try {
        const response = await fetch(logsUrl, { signal: controller.signal });
        const textData = await response.text();
        if (controller.signal.aborted) return;
        // Logs are JSONL there we have ensure they are parsed correctly and cleaned to be used
        const logsArray = parseJSONL(textData, {
          transform: (parsedLog) => {
            return {
              nodeId: parsedLog.nodeId,
              jobId,
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
        setFetchedLogs({ jobId, logs: logsArray });
      } catch (error) {
        if (controller.signal.aborted) return;
        console.error("Error fetching logs:", error);
        setFailedJobId(jobId);
      }
    })();

    return () => controller.abort();
  }, [jobId, logsUrl, hasFetched]);

  return (
    <LogsTable
      columns={columns}
      data={logs}
      isFetching={!logs.length || isFetchingLogsUrl}
      leadingActions={leadingActions}
      selectColumns
      showFiltering
    />
  );
};

export default LogsConsole;
