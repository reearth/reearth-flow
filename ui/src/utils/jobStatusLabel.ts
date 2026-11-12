import type { JobStatus } from "@flow/types";

export const getJobStatusLabel = (
  status: JobStatus | undefined,
  t: (key: string) => string,
) => {
  switch (status) {
    case "completed":
      return t("Completed");
    case "running":
      return t("Running");
    case "cancelled":
      return t("Cancelled");
    case "failed":
      return t("Failed");
    case "queued":
      return t("Queued");
    default:
      return t("Unknown");
  }
};
