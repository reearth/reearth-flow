import type { UserFacingLog } from "@flow/types";

export const logsToJsonl = (logs: UserFacingLog[]): string =>
  logs
    .map(({ timestamp, level, nodeId, nodeName, message }) =>
      JSON.stringify({ timestamp, level, nodeId, nodeName, message }),
    )
    .join("\n") + (logs.length ? "\n" : "");

export const downloadLogs = (jobId: string, logs: UserFacingLog[]) => {
  const blob = new Blob([logsToJsonl(logs)], { type: "application/x-ndjson" });
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = `job_${jobId}_logs.jsonl`;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
  URL.revokeObjectURL(url);
};
