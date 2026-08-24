import { CaretLeftIcon, XCircleIcon } from "@phosphor-icons/react";
import { useState } from "react";

import {
  Button,
  DiagnosticsTable,
  NodeExecutionsTable,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "@flow/components";
import { DetailsBox } from "@flow/features/common";
import LogsConsole from "@flow/features/LogsConsole";
import { useJobSubscriptionsSetup } from "@flow/hooks";
import { useT } from "@flow/lib/i18n";

import useHooks from "./hooks";

type Props = {
  jobId: string;
  accessToken: string;
};

const JobDetails: React.FC<Props> = ({ jobId, accessToken }) => {
  const t = useT();
  const [tabValue, setTabValue] = useState("log");

  useJobSubscriptionsSetup(accessToken, jobId);

  const {
    job,
    details,
    jobStatus,
    diagnostics,
    nodeExecutions,
    isFetchingNodeExecutions,
    handleBack,
    handleCancelJob,
  } = useHooks({
    jobId,
  });

  const failedNodes = job?.failedNodes;

  return (
    job && (
      <div className="flex flex-1 flex-col gap-4 px-6 pt-6 pb-2">
        <div className="flex justify-between">
          <Button size="icon" variant="ghost" onClick={handleBack}>
            <CaretLeftIcon />
          </Button>
          {(jobStatus === "queued" || jobStatus === "running") && (
            <Button variant="destructive" size="sm" onClick={handleCancelJob}>
              <XCircleIcon />
              {t("Cancel Job")}
            </Button>
          )}
        </div>
        <div className="w-full border-b" />
        <div className="mt-6 flex max-w-[1200px] flex-col">
          <DetailsBox collapsible title={t("Job Details")} content={details} />
        </div>
        <Tabs
          className="flex min-h-0 max-w-[1200px] flex-1 flex-col gap-2"
          value={tabValue}
          defaultValue="log"
          onValueChange={setTabValue}>
          <TabsList className="gap-2 self-start">
            <TabsTrigger value="log">{t("Log")}</TabsTrigger>
            <TabsTrigger value="diagnostics">
              {t("Diagnostics")}
              {diagnostics.length ? ` (${diagnostics.length})` : ""}
            </TabsTrigger>
            <TabsTrigger value="actions">{t("Actions")}</TabsTrigger>
          </TabsList>
          <TabsContent
            value="log"
            className="min-h-0 flex-1"
            keepMounted
            hidden={tabValue !== "log"}>
            <LogsConsole jobId={job.id} />
          </TabsContent>
          <TabsContent
            value="diagnostics"
            className="flex min-h-0 flex-1 flex-col gap-4 overflow-auto">
            {failedNodes?.length ? (
              <div className="rounded-md border border-destructive/50 p-4">
                <p className="mb-2 text-destructive">{t("Failed Actions")}</p>
                <DiagnosticsTable diagnostics={failedNodes} />
              </div>
            ) : null}
            <DiagnosticsTable
              diagnostics={diagnostics}
              isFetching={isFetchingNodeExecutions && !diagnostics.length}
              noResultsMessage={t(
                "No diagnostics reported for this job yet. Diagnostics appear while a job runs and are persisted once it finishes.",
              )}
            />
          </TabsContent>
          <TabsContent
            value="actions"
            className="flex min-h-0 flex-1 flex-col overflow-auto">
            <NodeExecutionsTable
              nodeExecutions={nodeExecutions ?? []}
              isFetching={isFetchingNodeExecutions && !nodeExecutions?.length}
              noResultsMessage={t(
                "No action executions reported for this job yet.",
              )}
            />
          </TabsContent>
        </Tabs>
      </div>
    )
  );
};

export { JobDetails };
