import type { GraphQLClient } from "graphql-request";
import { describe, expect, test, vi } from "vitest";

import {
  buildNodeDiagnosticsBatchQuery,
  fetchNodeDiagnosticsBatch,
} from "./nodeDiagnosticsBatch";

const row = (message: string) => ({ message }) as never;

describe("buildNodeDiagnosticsBatchQuery", () => {
  test("aliases one field per bucket, with a variable each", () => {
    const query = buildNodeDiagnosticsBatchQuery(2);

    expect(query).toContain("$n0: String!, $n1: String!");
    expect(query).toContain("b0: nodeDiagnostics(nodeId: $n0)");
    expect(query).toContain("b1: nodeDiagnostics(nodeId: $n1)");
  });

  test("stays valid with no buckets at all", () => {
    // A project with no nodes yet still renders the panel, and a trailing comma
    // in the variable list would make the document unparseable.
    const query = buildNodeDiagnosticsBatchQuery(0);

    expect(query).toContain("query GetNodeDiagnosticsBatch($jobId: ID!)");
    expect(query).not.toContain(",");
  });

  test("names aliases positionally, never from the node id", () => {
    // Node ids are UUIDs and often start with a digit, which is not a legal
    // GraphQL alias — using them directly would produce an invalid document.
    expect(buildNodeDiagnosticsBatchQuery(1)).toContain("b0:");
  });
});

describe("buildNodeDiagnosticsBatchQuery - failedNodes", () => {
  test("selects failedNodes here rather than on the shared Job fragment", () => {
    // failedNodes has its own per-job resolver, so carrying it on the fragment
    // that GetJobs spreads made the workspace list resolve and transfer a
    // diagnostics payload for every row, none of which the list renders.
    expect(buildNodeDiagnosticsBatchQuery(0)).toContain(
      "failedNodes { ...Diagnostic }",
    );
  });
});

describe("fetchNodeDiagnosticsBatch", () => {
  const clientReturning = (data: unknown) =>
    ({ request: vi.fn().mockResolvedValue(data) }) as unknown as GraphQLClient;

  test("flattens every bucket's rows and keeps failedNodes separate", () => {
    // They stay apart because the caller filters fatal rows out of the buckets:
    // failedNodes already holds every fatal row for the job.
    const client = clientReturning({
      job: {
        id: "job-1",
        failedNodes: [row("terminal")],
        b0: [row("job level")],
        b1: [row("from node-1")],
      },
    });

    return expect(
      fetchNodeDiagnosticsBatch(client, "job-1", ["", "node-1"]),
    ).resolves.toEqual({
      failedNodes: [row("terminal")],
      bucketRows: [row("job level"), row("from node-1")],
    });
  });

  test("treats a null bucket as empty rather than throwing", () => {
    // nodeDiagnostics is nullable, and a node that reported nothing comes back
    // null rather than as an empty list.
    const client = clientReturning({ job: { id: "job-1", b0: null } });

    return expect(
      fetchNodeDiagnosticsBatch(client, "job-1", [""]),
    ).resolves.toEqual({ failedNodes: [], bucketRows: [] });
  });

  test("collapses duplicate ids so their rows are not counted twice", async () => {
    // The caller concatenates the job-level id with the workflow's node ids; if
    // the graph ever contained the empty id the same bucket would be requested
    // twice, and a Diagnostic carries no id to dedupe by afterwards.
    const client = clientReturning({ job: { id: "job-1", b0: [row("once")] } });

    const { bucketRows } = await fetchNodeDiagnosticsBatch(client, "job-1", [
      "",
      "",
    ]);

    expect(bucketRows).toEqual([row("once")]);
    expect(client.request).toHaveBeenCalledWith(expect.any(String), {
      jobId: "job-1",
      n0: "",
    });
  });

  test("survives a job that came back null", () => {
    const client = clientReturning({ job: null });

    return expect(
      fetchNodeDiagnosticsBatch(client, "job-1", ["node-1"]),
    ).resolves.toEqual({ failedNodes: [], bucketRows: [] });
  });
});
