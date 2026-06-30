import type { NodeReport, NodeSchemaMeta, SchemaReport } from "@flow/types";

/** Artifact filename the engine writes the probe result to. */
export const SCHEMA_REPORT_FILENAME = "schema-report.json";

/**
 * Pick the schema-report artifact out of a completed preview job's
 * `outputURLs`. The server writes it to
 * `artifacts/<jobId>/schema/schema-report.json`.
 */
export const findSchemaReportUrl = (urls?: string[]): string | undefined => {
  if (!urls?.length) return undefined;
  return (
    urls.find((url) => url.includes(SCHEMA_REPORT_FILENAME)) ??
    urls.find((url) => url.includes("/schema/"))
  );
};

export const fetchSchemaReport = async (url: string): Promise<SchemaReport> => {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`Failed to fetch schema report (${response.status})`);
  }
  return (await response.json()) as SchemaReport;
};

/** Map a node's report entry into the metadata persisted on the node. */
export const toNodeSchemaMeta = (
  nodeReport: NodeReport,
  sampleSize: number,
  sampledAt: string,
): NodeSchemaMeta => ({
  ports: nodeReport.ports,
  sampleSize,
  sampledAt,
  note: nodeReport.note,
});
