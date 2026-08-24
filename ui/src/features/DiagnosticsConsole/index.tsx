import { useMemo, useState } from "react";

import {
  DiagnosticsTable,
  NodeExecutionsTable,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "@flow/components";
import { useJob } from "@flow/lib/gql/job";
import { useT } from "@flow/lib/i18n";
import { type Diagnostic, compareDiagnosticSeverity } from "@flow/types";

type Props = {
  jobId: string;
  /**
   * Whether the job is still running. Diagnostics and feature counts are absent
   * from the status subscription payloads, so they can only be kept current by
   * polling while the run is live.
   */
  isJobActive?: boolean;
};

/**
 * Structured engine diagnostics for one job, read from its node executions.
 *
 * Deliberately sourced from `nodeExecutions[].diagnostics` rather than
 * `job.failedNodes`: the former is served live (from a TTL-bound cache while
 * the job runs, merged with the persisted rows once it finishes), whereas
 * `failedNodes` is persisted at completion and so is empty for the whole of a
 * run. A console that has to be useful mid-run cannot be built on it.
 */
const DiagnosticsConsole: React.FC<Props> = ({ jobId, isJobActive }) => {
  const t = useT();
  const [tabValue, setTabValue] = useState("diagnostics");

  const { useGetNodeExecutions } = useJob();

  const { nodeExecutions, isFetching } = useGetNodeExecutions(
    jobId,
    isJobActive,
  );

  const diagnostics: Diagnostic[] = useMemo(
    () =>
      (nodeExecutions ?? [])
        .flatMap((nodeExecution) => nodeExecution.diagnostics ?? [])
        .sort(compareDiagnosticSeverity),
    [nodeExecutions],
  );

  return (
    <Tabs
      className="flex h-full min-h-0 flex-col gap-2"
      value={tabValue}
      defaultValue="diagnostics"
      onValueChange={setTabValue}>
      <TabsList className="gap-2 self-start">
        <TabsTrigger value="diagnostics">
          {t("Diagnostics")}
          {diagnostics.length ? ` (${diagnostics.length})` : ""}
        </TabsTrigger>
        <TabsTrigger value="actions">{t("Actions")}</TabsTrigger>
      </TabsList>
      <TabsContent
        value="diagnostics"
        className="flex min-h-0 flex-1 flex-col overflow-auto">
        <DiagnosticsTable
          diagnostics={diagnostics}
          isFetching={isFetching && !diagnostics.length}
          noResultsMessage={t(
            "No diagnostics reported for this run yet. Diagnostics appear while a run is in progress and are persisted once it finishes.",
          )}
        />
      </TabsContent>
      <TabsContent
        value="actions"
        className="flex min-h-0 flex-1 flex-col overflow-auto">
        <NodeExecutionsTable
          nodeExecutions={nodeExecutions ?? []}
          isFetching={isFetching && !nodeExecutions?.length}
          noResultsMessage={t(
            "No action executions reported for this run yet.",
          )}
        />
      </TabsContent>
    </Tabs>
  );
};

export default DiagnosticsConsole;
