import { useRouter } from "@tanstack/react-router";
import { useCallback, useMemo } from "react";

import { DetailsBoxContent } from "@flow/features/common";
import { useJob } from "@flow/lib/gql/job";
import { useJobStatus } from "@flow/lib/gql/job/useSubscriptions";
import { useT } from "@flow/lib/i18n";
import { formatTimestamp } from "@flow/utils";

export default ({ jobId }: { jobId: string }) => {
  const t = useT();
  const { navigate } = useRouter();

  const { useGetJob, useJobCancel } = useJob();

  const { data: jobStatus, isLoading, error } = useJobStatus(jobId);

  const statusValue = isLoading
    ? t("Loading...")
    : error
      ? t("Error")
      : jobStatus;

  const { job } = useGetJob(jobId);

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
              value: statusValue || job.status,
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
                job.status === "completed"
                  ? formatTimestamp(job.completedAt)
                  : t("N/A"),
            },
            {
              id: "outputURLs",
              name: t("Output URLs"),
              value: job.outputURLs || t("N/A"),
              type: job.outputURLs ? "link" : undefined,
            },
          ]
        : undefined,
    [t, job, statusValue],
  );
  return {
    job,
    details,
    statusValue,
    handleCancelJob,
    handleBack,
  };
};
