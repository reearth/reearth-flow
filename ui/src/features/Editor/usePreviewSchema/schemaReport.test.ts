import type { NodeReport, SchemaReport } from "@flow/types";

import {
  fetchSchemaReport,
  findSchemaReportUrl,
  toNodeSchemaMeta,
} from "./schemaReport";

describe("findSchemaReportUrl", () => {
  test("returns undefined for empty/missing urls", () => {
    expect(findSchemaReportUrl(undefined)).toBeUndefined();
    expect(findSchemaReportUrl([])).toBeUndefined();
  });

  test("picks the schema-report.json artifact", () => {
    const urls = [
      "https://example.com/artifacts/job-1/other.json",
      "https://example.com/artifacts/job-1/schema/schema-report.json",
    ];
    expect(findSchemaReportUrl(urls)).toBe(urls[1]);
  });

  test("falls back to a /schema/ artifact", () => {
    const urls = ["https://example.com/artifacts/job-1/schema/report.bin"];
    expect(findSchemaReportUrl(urls)).toBe(urls[0]);
  });
});

describe("toNodeSchemaMeta", () => {
  test("maps a node report into persisted metadata", () => {
    const nodeReport: NodeReport = {
      name: "reader",
      note: "sampled 2",
      ports: {
        default: {
          open: false,
          fields: [{ name: "id", type: "String" }],
        },
      },
    };

    expect(
      toNodeSchemaMeta(nodeReport, 10, "2026-06-17T00:00:00.000Z"),
    ).toEqual({
      ports: nodeReport.ports,
      sampleSize: 10,
      sampledAt: "2026-06-17T00:00:00.000Z",
      note: "sampled 2",
      status: "complete",
    });
  });
});

describe("fetchSchemaReport", () => {
  const report: SchemaReport = {
    version: 1,
    sampleSize: 10,
    nodes: {
      "node-1": {
        name: "reader",
        ports: {
          default: {
            open: false,
            fields: [{ name: "id", type: "String" }],
          },
        },
      },
    },
  };

  test("parses a successful response", async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => report,
    });
    vi.stubGlobal("fetch", fetchMock);

    await expect(
      fetchSchemaReport("https://example.com/r.json"),
    ).resolves.toEqual(report);
    expect(fetchMock).toHaveBeenCalledWith("https://example.com/r.json");

    vi.unstubAllGlobals();
  });

  test("throws on a non-ok response", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue({ ok: false, status: 404 }),
    );

    await expect(
      fetchSchemaReport("https://example.com/missing.json"),
    ).rejects.toThrow("404");

    vi.unstubAllGlobals();
  });
});
