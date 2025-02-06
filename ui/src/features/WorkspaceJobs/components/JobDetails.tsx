import { CaretLeft } from "@phosphor-icons/react";
import { useRouter } from "@tanstack/react-router";
import { useCallback, useEffect, useMemo, useState } from "react";

import { Button, Loading } from "@flow/components";
import { config } from "@flow/config";
import { DetailsBox, DetailsBoxContent } from "@flow/features/common";
import { LogsConsole } from "@flow/features/Editor/components/BottomPanel/components";
import { useT } from "@flow/lib/i18n";
import type { Job, Log } from "@flow/types";

type Props = {
  selectedJob?: Job;
};

const JobDetails: React.FC<Props> = ({ selectedJob }) => {
  const t = useT();
  const { history } = useRouter();

  const handleBack = useCallback(() => history.go(-1), [history]); // Go back to previous page
  const [logs, setLogs] = useState<Log[] | null>(null);
  const [isFetching, setIsFetching] = useState<boolean>(false);
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
              value: selectedJob.startedAt,
            },
            {
              id: "completedAt",
              name: t("Completed At"),
              value: selectedJob.completedAt || t("N/A"),
            },
          ]
        : undefined,
    [t, selectedJob],
  );
  // Note: This is only temporary and will be replaced with a proper log fetching mechanism when the API is ready @Billcookie
  const getAllLogs = useCallback(async () => {
    const BASE_URL = config().api;
    if (!selectedJob) return;
    setIsFetching(true);
    try {
      const response = await fetch(
        `${BASE_URL}/artifacts/${selectedJob.id}/action-log/all.log`,
      );
      const textData = await response.text();
      const logsArray = textData
        .split("\n")
        .filter((line) => line.trim() !== "")
        .map((line) => {
          try {
            const parsed = JSON.parse(line);
            return {
              workflowId: selectedJob.workspaceId,
              jobId: selectedJob.id,
              msg: parsed.msg,
              ts: parsed.ts,
              level: parsed.level,
            };
          } catch (error) {
            console.error("Failed to parse log line:", line, error);
            return null;
          }
        })
        .filter((log) => log !== null);
      setLogs(logsArray);
    } catch (error) {
      console.error("Error fetching logs:", error);
      setIsFetching(false);
    } finally {
      setIsFetching(false);
    }
  }, [selectedJob, setIsFetching]);

  useEffect(() => {
    getAllLogs();
  }, [getAllLogs]);

  return (
    selectedJob && (
      <div className="flex flex-1 flex-col gap-4 px-6 pb-2 pt-6">
        <div className="flex justify-between">
          <Button size="icon" variant="ghost" onClick={handleBack}>
            <CaretLeft />
          </Button>
        </div>
        <div className="w-full border-b" />
        <div className="mt-6 flex max-w-[1200px] flex-col gap-6">
          <DetailsBox title={t("Job Details")} content={details} />
        </div>
        <div className="mt-6 min-h-0 max-w-[1200px] flex-1">
          {!isFetching && logs && <LogsConsole data={logs} />}
          {isFetching && <Loading />}
        </div>
      </div>
    )
  );
};

export { JobDetails };
