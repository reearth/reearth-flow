import * as zlib from "node:zlib";

import { describe, expect, test } from "vitest";

import { FeatureRowsHost } from "./host";
import type { StatusMessage, WorkerResponse } from "./protocol";

const encoder = new TextEncoder();

const feature = (n: number, attributes: Record<string, unknown> = { n }) =>
  JSON.stringify({
    id: `f-${n}`,
    attributes,
    geometry: {
      Euclidean2D: { Point: { frame: { Crs: 4326 }, position: [35.6, 139.7] } },
    },
  });

const jsonl = (count: number) =>
  Array.from({ length: count }, (_, n) => feature(n)).join("\n") + "\n";

function streamOf(bytes: Uint8Array, pieces = 4) {
  const size = Math.ceil(bytes.length / pieces);
  let offset = 0;
  return new ReadableStream<Uint8Array>({
    pull(controller) {
      if (offset >= bytes.length) {
        controller.close();
        return;
      }
      controller.enqueue(bytes.slice(offset, offset + size));
      offset += size;
    },
  });
}

function setup(files: Record<string, Uint8Array>) {
  const messages: WorkerResponse[] = [];
  const requested: string[] = [];
  const aborted: string[] = [];
  const host = new FeatureRowsHost({
    post: (message) => messages.push(message),
    fetch: (async (input: string, init?: RequestInit) => {
      const url = String(input);
      requested.push(url);
      init?.signal?.addEventListener("abort", () => aborted.push(url));
      const bytes = files[url];
      if (!bytes) return new Response(null, { status: 404 });
      return new Response(streamOf(bytes));
    }) as typeof fetch,
  });

  const statuses = (url: string) =>
    messages
      .filter(
        (m): m is { type: "status"; status: StatusMessage } =>
          m.type === "status" && m.status.url === url,
      )
      .map((m) => m.status);
  const lastStatus = (url: string) => statuses(url).at(-1);

  const reply = <T extends WorkerResponse["type"]>(type: T, id: number) =>
    messages.find(
      (m) => m.type === type && "id" in m && m.id === id,
    ) as Extract<WorkerResponse, { type: T }>;

  return { host, messages, requested, aborted, statuses, lastStatus, reply };
}

describe("FeatureRowsHost", () => {
  test("loads every row of a plain JSONL file, with no cap", async () => {
    const { host, lastStatus, statuses } = setup({
      "a.jsonl": encoder.encode(jsonl(5000)),
    });
    host.handle({ type: "open", url: "a.jsonl" });
    await host.settled("a.jsonl");

    expect(lastStatus("a.jsonl")).toMatchObject({
      rowCount: 5000,
      totalRows: 5000,
      viewCount: 5000,
      done: true,
      truncated: false,
      error: null,
      geometryType: expect.any(String),
    });
    const withColumns = statuses("a.jsonl").filter((s) => s.columns);
    expect(withColumns.at(-1)?.columns).toEqual(["n"]);
  });

  test.skipIf(typeof zlib.zstdCompressSync !== "function")(
    "decompresses a .jsonl.zst file",
    async () => {
      const compressed = new Uint8Array(
        zlib.zstdCompressSync(Buffer.from(jsonl(3000))),
      );
      const { host, lastStatus } = setup({ "b.jsonl.zst?sig=1": compressed });
      host.handle({ type: "open", url: "b.jsonl.zst?sig=1" });
      await host.settled("b.jsonl.zst?sig=1");
      expect(lastStatus("b.jsonl.zst?sig=1")).toMatchObject({
        rowCount: 3000,
        done: true,
        error: null,
      });
    },
  );

  test("returns a window of rows with their attributes", async () => {
    const { host, reply } = setup({ "a.jsonl": encoder.encode(jsonl(300)) });
    host.handle({ type: "open", url: "a.jsonl" });
    await host.settled("a.jsonl");

    host.handle({
      type: "rows",
      id: 1,
      url: "a.jsonl",
      start: 250,
      count: 100,
    });
    const rows = reply("rows", 1).rows;
    expect(rows).toHaveLength(50);
    expect(rows[0]).toEqual({ row: 250, id: "f-250", attributes: { n: 250 } });
  });

  test("returns a row's raw line for the details panel", async () => {
    const { host, reply } = setup({ "a.jsonl": encoder.encode(jsonl(10)) });
    host.handle({ type: "open", url: "a.jsonl" });
    await host.settled("a.jsonl");

    host.handle({ type: "line", id: 7, url: "a.jsonl", row: 3 });
    expect(JSON.parse(reply("line", 7).text ?? "")).toMatchObject({
      id: "f-3",
    });
  });

  test("searches, and maps rows to their place in the results", async () => {
    const lines = [
      feature(0, { city: "Tokyo" }),
      feature(1, { city: "Osaka" }),
      feature(2, { city: "tokyo" }),
    ].join("\n");
    const { host, lastStatus, reply } = setup({
      "a.jsonl": encoder.encode(lines),
    });
    host.handle({ type: "open", url: "a.jsonl" });
    await host.settled("a.jsonl");

    host.handle({ type: "search", url: "a.jsonl", term: " TOKYO " });
    await host.settled("a.jsonl");
    expect(lastStatus("a.jsonl")).toMatchObject({
      viewCount: 2,
      rowCount: 3,
      searchTerm: "TOKYO",
      searching: false,
    });

    host.handle({ type: "rows", id: 1, url: "a.jsonl", start: 0, count: 10 });
    expect(reply("rows", 1).rows.map((r) => r.row)).toEqual([0, 2]);

    host.handle({ type: "viewIndex", id: 2, url: "a.jsonl", row: 2 });
    expect(reply("viewIndex", 2).index).toBe(1);

    host.handle({ type: "search", url: "a.jsonl", term: "" });
    expect(lastStatus("a.jsonl")).toMatchObject({ viewCount: 3 });
  });

  test("rows say which search term they were filtered by", async () => {
    const lines = [
      feature(0, { name: "ab" }),
      feature(1, { name: "abc" }),
    ].join("\n");
    const { host, reply } = setup({ "a.jsonl": encoder.encode(lines) });
    host.handle({ type: "open", url: "a.jsonl" });
    await host.settled("a.jsonl");

    host.handle({ type: "search", url: "a.jsonl", term: "ab" });
    await host.settled("a.jsonl");
    host.handle({ type: "search", url: "a.jsonl", term: "abc" });
    host.handle({ type: "rows", id: 1, url: "a.jsonl", start: 0, count: 10 });
    await host.settled("a.jsonl");

    expect(reply("rows", 1).searchTerm).toBe("abc");
  });

  test("clear drops every cached file, so the next open fetches again", async () => {
    const { host, requested, reply } = setup({
      "a.jsonl": encoder.encode(jsonl(10)),
    });
    host.handle({ type: "open", url: "a.jsonl" });
    await host.settled("a.jsonl");
    host.handle({ type: "clear" });

    host.handle({ type: "rows", id: 1, url: "a.jsonl", start: 0, count: 10 });
    expect(reply("rows", 1).rows).toEqual([]);

    host.handle({ type: "open", url: "a.jsonl" });
    await host.settled("a.jsonl");
    expect(requested).toEqual(["a.jsonl", "a.jsonl"]);
  });

  test("switching ports cancels the unfinished load and stops its statuses", async () => {
    const { host, aborted, statuses } = setup({
      "a.jsonl": encoder.encode(jsonl(2000)),
      "b.jsonl": encoder.encode(jsonl(10)),
    });
    host.handle({ type: "open", url: "a.jsonl" });
    host.handle({ type: "open", url: "b.jsonl" });
    const before = statuses("a.jsonl").length;
    await host.settled("b.jsonl");

    expect(aborted).toEqual(["a.jsonl"]);
    expect(statuses("a.jsonl")).toHaveLength(before);
  });

  test("going back to a finished port reuses it without fetching again", async () => {
    const { host, requested, lastStatus } = setup({
      "a.jsonl": encoder.encode(jsonl(10)),
      "b.jsonl": encoder.encode(jsonl(20)),
    });
    host.handle({ type: "open", url: "a.jsonl" });
    await host.settled("a.jsonl");
    host.handle({ type: "open", url: "b.jsonl" });
    await host.settled("b.jsonl");
    host.handle({ type: "open", url: "a.jsonl" });

    expect(requested).toEqual(["a.jsonl", "b.jsonl"]);
    expect(lastStatus("a.jsonl")).toMatchObject({
      rowCount: 10,
      done: true,
      columns: ["n"],
    });
  });

  test("reports a failed download", async () => {
    const { host, lastStatus } = setup({});
    host.handle({ type: "open", url: "missing.jsonl" });
    await host.settled("missing.jsonl");
    expect(lastStatus("missing.jsonl")).toMatchObject({
      done: true,
      error: expect.stringContaining("404"),
    });
  });
});
