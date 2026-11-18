import type { NodeReport, SchemaReport } from "@flow/types";

import {
  applySchemaPatch,
  fetchSchemaReport,
  findSchemaReportUrl,
  getNodeReportFailure,
  resolveNodeSchema,
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

describe("getNodeReportFailure", () => {
  test("returns undefined when there is no note (clean success)", () => {
    const nodeReport: NodeReport = {
      name: "reader",
      ports: {
        default: { open: false, fields: [{ name: "id", type: "String" }] },
      },
    };
    expect(getNodeReportFailure(nodeReport)).toBeUndefined();
  });

  test("returns the note when the reader sampled nothing (data error)", () => {
    const nodeReport: NodeReport = {
      name: "reader",
      note: "source run failed: CSV column mismatch",
      ports: { default: { open: true, fields: [] } },
    };
    expect(getNodeReportFailure(nodeReport)).toBe(
      "source run failed: CSV column mismatch",
    );
  });

  test("ignores a note when fields were inferred (open schema, not a failure)", () => {
    const nodeReport: NodeReport = {
      name: "reader",
      note: "open schema",
      ports: {
        default: { open: true, fields: [{ name: "id", type: "String" }] },
      },
    };
    expect(getNodeReportFailure(nodeReport)).toBeUndefined();
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

describe("applySchemaPatch", () => {
  test("drops the job id once the probe is no longer running", () => {
    expect(
      applySchemaPatch(
        { ports: {}, status: "running", jobId: "job-1" },
        { status: "failed" },
      ),
    ).toEqual({ ports: {}, status: "failed" });
  });

  test("clears the schema when there is no patch", () => {
    expect(applySchemaPatch({ ports: {} }, undefined)).toBeUndefined();
  });
});

describe("resolveNodeSchema", () => {
  const outcome = { ports: {}, status: "complete" as const };

  test("shows the local outcome of a running probe", () => {
    expect(
      resolveNodeSchema(
        { ports: {}, status: "running", jobId: "job-1" },
        { "job-1": outcome },
      ),
    ).toBe(outcome);
  });

  test("ignores outcomes of other jobs and of finished probes", () => {
    const running = { ports: {}, status: "running" as const, jobId: "job-2" };
    expect(resolveNodeSchema(running, { "job-1": outcome })).toBe(running);
    const failed = { ports: {}, status: "failed" as const };
    expect(resolveNodeSchema(failed, { "job-1": outcome })).toBe(failed);
  });
});
