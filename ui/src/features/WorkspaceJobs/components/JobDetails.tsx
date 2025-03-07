import { CaretLeft, XCircle } from "@phosphor-icons/react";
import { useRouter } from "@tanstack/react-router";
import { useCallback, useEffect, useMemo, useState } from "react";

import { Button, LoadingSkeleton } from "@flow/components";
import { DetailsBox, DetailsBoxContent } from "@flow/features/common";
import { LogsConsole } from "@flow/features/Editor/components/BottomPanel/components";
import { useLogs } from "@flow/lib/gql/realTimeLogs/useSubscriptions";
import { useT } from "@flow/lib/i18n";
import type { Job, Log } from "@flow/types";
import { formatTimestamp } from "@flow/utils";
import { parseJSONL } from "@flow/utils/parseJsonL";

type Props = {
  selectedJob?: Job;
  onJobCancel?: () => void;
};

const JobDetails: React.FC<Props> = ({ selectedJob, onJobCancel }) => {
  const t = useT();
  const { navigate } = useRouter();

  const handleBack = useCallback(
    () =>
      navigate({
        to: `/workspaces/${selectedJob?.workspaceId}/jobs/all`,
      }),
    [navigate, selectedJob?.workspaceId],
  );

  const [urlLogs, setUrlLogs] = useState<Log[] | null>(null);
  const [liveLogs, setLiveLogs] = useState<Log[] | null | undefined>(undefined);
  const [isFetchingLogsUrl, setIsFetchingLogsUrl] = useState<boolean>(false);
  const details: DetailsBoxContent[] | undefined = useMemo(
    () =>
      selectedJob
        ? [
            {
              id: "id",
              name: t("ID"),
              value: selectedJob.id,
            },
            {
              id: "deploymentId",
              name: t("Deployment ID"),
              value: selectedJob.deploymentId,
            },
            {
              id: "status",
              name: t("Status"),
              value: selectedJob.status,
            },
            {
              id: "startedAt",
              name: t("Started At"),
              value: formatTimestamp(selectedJob.startedAt) || t("N/A"),
            },

            {
              id: "completedAt",
              name: t("Completed At"),
              value:
                selectedJob.status === "completed"
                  ? formatTimestamp(selectedJob.completedAt)
                  : t("N/A"),
            },
            {
              id: "outputURLs",
              name: t("Output URLs"),
              value: selectedJob.outputURLs || t("N/A"),
              type: selectedJob.outputURLs ? "link" : undefined,
            },
          ]
        : undefined,
    [t, selectedJob],
  );

  const { data, isLoading: isFetchingLiveLogs } = useLogs(
    selectedJob?.id ?? "",
  );

  const getLogsFromUrl = useCallback(async () => {
    if (
      !selectedJob ||
      !selectedJob.logsURL ||
      selectedJob.status !== "completed"
    )
      return;
    setIsFetchingLogsUrl(true);
    try {
      const response = await fetch(selectedJob.logsURL);
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
            jobId: selectedJob.id,
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
      setUrlLogs(logsArray);
    } catch (error) {
      console.error("Error fetching logs:", error);
    } finally {
      setIsFetchingLogsUrl(false);
    }
  }, [selectedJob, setIsFetchingLogsUrl]);

  useEffect(() => {
    if (selectedJob?.logsURL) {
      getLogsFromUrl();
    }

    if (!data) return;
    setLiveLogs(data);
  }, [getLogsFromUrl, data, selectedJob?.logsURL]);

  return (
    selectedJob && (
      <div className="flex flex-1 flex-col gap-4 px-6 pb-2 pt-6">
        <div className="flex justify-between">
          <Button size="icon" variant="ghost" onClick={handleBack}>
            <CaretLeft />
          </Button>
          {(selectedJob.status === "queued" ||
            selectedJob.status === "running") && (
            <Button variant="destructive" size="sm" onClick={onJobCancel}>
              <XCircle />
              {t("Cancel Job")}
            </Button>
          )}
        </div>
        <div className="w-full border-b" />
        <div className="mt-6 flex max-w-[1200px] flex-col gap-6">
          <DetailsBox collapsible title={t("Job Details")} content={details} />
        </div>
        <div className="mt-6 min-h-0 max-w-[1200px] flex-1">
          {selectedJob.status !== "completed" && (
            <>
              {isFetchingLiveLogs ? (
                <LoadingSkeleton />
              ) : liveLogs && liveLogs.length > 0 ? (
                <LogsConsole data={liveLogs} />
              ) : null}
            </>
          )}
          {selectedJob.status === "completed" && (
            <>
              {isFetchingLogsUrl ? (
                <LoadingSkeleton />
              ) : urlLogs && urlLogs.length > 0 ? (
                <LogsConsole data={urlLogs} />
              ) : null}
            </>
          )}
        </div>
      </div>
    )
  );
};

export { JobDetails };
