import { useRouter } from "@tanstack/react-router";
import { useCallback, useEffect, useMemo } from "react";

import { DetailsBoxContent } from "@flow/features/common";
import { useJob } from "@flow/lib/gql/job";
import { useSubscription } from "@flow/lib/gql/subscriptions/useSubscription";
import { useT } from "@flow/lib/i18n";
import { type Diagnostic, compareDiagnosticSeverity } from "@flow/types";
import { formatTimestamp } from "@flow/utils";

export default ({ jobId }: { jobId: string }) => {
  const t = useT();
  const { navigate } = useRouter();

  const { useGetJob, useGetNodeExecutions, useJobCancel } = useJob();

  const { data: jobStatus } = useSubscription("GetSubscribedJobStatus", jobId);

  const { job, refetch } = useGetJob(jobId);

  const currentStatus = jobStatus ?? job?.status;
  const isJobActive = currentStatus === "running" || currentStatus === "queued";

  const {
    nodeExecutions,
    isFetching: isFetchingNodeExecutions,
    refetch: refetchNodeExecutions,
  } = useGetNodeExecutions(jobId, isJobActive);

  // The status subscriptions carry the status enum and nothing else, so a
  // status change is only a cue to go re-read the rows that actually hold the
  // diagnostics and feature counts. `failedNodes` in particular is persisted at
  // completion, so the job itself has to be re-read too.
  useEffect(() => {
    if (!jobStatus) return;
    refetch();
    refetchNodeExecutions();
  }, [jobStatus, refetch, refetchNodeExecutions]);

  const diagnostics: Diagnostic[] = useMemo(
    () =>
      (nodeExecutions ?? [])
        .flatMap((nodeExecution) => nodeExecution.diagnostics ?? [])
        .sort(compareDiagnosticSeverity),
    [nodeExecutions],
  );

  // Poll for outputURLs after job completes (they are generated asynchronously),
  // and for failedNodes after it fails: those are persisted at completion, so
  // the status event can land before the write does.
  useEffect(() => {
    const awaitingOutputURLs = jobStatus === "completed" && !job?.outputURLs;
    const awaitingFailedNodes = jobStatus === "failed" && !job?.failedNodes;

    if (job && (awaitingOutputURLs || awaitingFailedNodes)) {
      const pollInterval = setInterval(() => {
        refetch();
      }, 3000);

      const timeout = setTimeout(() => {
        clearInterval(pollInterval);
      }, 30000);

      return () => {
        clearInterval(pollInterval);
        clearTimeout(timeout);
      };
    }
  }, [jobStatus, job, refetch]);

  const handleCancelJob = useCallback(async () => {
    await useJobCancel(jobId);
  }, [jobId, useJobCancel]);

  const handleBack = useCallback(
    () =>
      navigate({
        to: `/workspaces/${job?.workspaceId}/jobs`,
      }),
    [job?.workspaceId, navigate],
  );

  const details: DetailsBoxContent[] | undefined = useMemo(
    () =>
      job
        ? [
            {
              id: "id",
              name: t("ID"),
              value: job.id,
            },
            {
              id: "deploymentId",
              name: t("Deployment ID"),
              value: job.deploymentId || t("N/A"),
            },
            {
              id: "deploymentDescription",
              name: t("Deployment"),
              value: job.deploymentDescription || t("N/A"),
            },
            {
              id: "status",
              name: t("Status"),
              value: jobStatus || job.status,
            },
            {
              id: "startedAt",
              name: t("Started At"),
              value: formatTimestamp(job.startedAt) || t("N/A"),
            },

            {
              id: "completedAt",
              name: t("Completed At"),
              value:
                job.status === "completed" ||
                job.status === "failed" ||
                job.status === "cancelled"
                  ? formatTimestamp(job.completedAt)
                  : t("N/A"),
            },
            {
              id: "outputURLs",
              name: t("Output URLs"),
              value: job.outputURLs || t("N/A"),
              type: job.outputURLs ? "link" : undefined,
            },
            // Only worth surfacing when something was actually lost: a
            // non-zero count means the diagnostics below are incomplete.
            ...(job.droppedEventCount
              ? [
                  {
                    id: "droppedEventCount",
                    name: t("Dropped Diagnostics"),
                    value: job.droppedEventCount.toLocaleString(),
                  },
                ]
              : []),
          ]
        : undefined,
    [t, job, jobStatus],
  );
  return {
    job,
    details,
    jobStatus,
    diagnostics,
    nodeExecutions,
    isFetchingNodeExecutions,
    handleCancelJob,
    handleBack,
  };
};
