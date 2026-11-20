import { describe, expect, test } from "vitest";

import { FeatureRowsClient } from "./client";
import type { WorkerLike } from "./client";
import type {
  FeatureRowsStatus,
  StatusMessage,
  WorkerRequest,
  WorkerResponse,
} from "./protocol";

const statusMessage = (
  status: Partial<StatusMessage> & { url: string },
): MessageEvent<WorkerResponse> =>
  new MessageEvent("message", {
    data: {
      type: "status",
      status: {
        rowCount: 0,
        totalRows: 0,
        viewCount: 0,
        searchTerm: "",
        searching: false,
        done: false,
        truncated: false,
        skippedLines: 0,
        geometryType: null,
        error: null,
        ...status,
      },
    },
  });

function fakeWorker() {
  const sent: WorkerRequest[] = [];
  let terminated = false;
  const worker: WorkerLike = {
    postMessage: (message) => sent.push(message),
    onmessage: null,
    onerror: null,
    onmessageerror: null,
    terminate: () => {
      terminated = true;
    },
  };
  return { worker, sent, isTerminated: () => terminated };
}

describe("FeatureRowsClient", () => {
  test("a crashed worker rejects pending requests and reports an error", async () => {
    const workers: ReturnType<typeof fakeWorker>[] = [];
    const client = new FeatureRowsClient(() => {
      const fake = fakeWorker();
      workers.push(fake);
      return fake.worker;
    });
    const statuses: FeatureRowsStatus[] = [];
    client.subscribe("a.jsonl", (status) => statuses.push(status));
    client.open("a.jsonl");
    const pending = client.rows("a.jsonl", 0, 100);

    workers[0].worker.onerror?.(new ErrorEvent("error", { message: "boom" }));

    await expect(pending).rejects.toThrow("boom");
    expect(workers[0].isTerminated()).toBe(true);
    expect(statuses.at(-1)).toMatchObject({
      url: "a.jsonl",
      rowCount: 0,
      viewCount: 0,
      done: true,
      error: "boom",
    });

    client.open("a.jsonl");
    expect(workers).toHaveLength(2);
  });

  test("rows resolve with the search term the worker filtered by", async () => {
    const fake = fakeWorker();
    const client = new FeatureRowsClient(() => fake.worker);
    const pending = client.rows("a.jsonl", 0, 100);
    const request = fake.sent[0];
    if (request.type !== "rows") throw new Error("expected a rows request");

    fake.worker.onmessage?.(
      new MessageEvent("message", {
        data: { type: "rows", id: request.id, searchTerm: "abc", rows: [] },
      }),
    );

    await expect(pending).resolves.toEqual({ searchTerm: "abc", rows: [] });
  });

  test("clear ends the worker, ignores its late messages, and reopens what is shown", async () => {
    const workers: ReturnType<typeof fakeWorker>[] = [];
    const client = new FeatureRowsClient(() => {
      const fake = fakeWorker();
      workers.push(fake);
      return fake.worker;
    });
    const statuses: FeatureRowsStatus[] = [];
    client.subscribe("a.jsonl", (status) => statuses.push(status));
    client.open("a.jsonl");
    const pending = client.line("a.jsonl", 0);
    const old = workers[0];

    client.clear();
    await expect(pending).rejects.toThrow();
    expect(old.isTerminated()).toBe(true);
    expect(client.status("a.jsonl")).toBeUndefined();

    old.worker.onmessage?.(statusMessage({ url: "a.jsonl", rowCount: 5 }));
    expect(statuses).toHaveLength(0);

    await Promise.resolve();
    expect(workers).toHaveLength(2);
    expect(workers[1].sent).toEqual([{ type: "open", url: "a.jsonl" }]);
  });

  test("a request whose send throws is rejected and forgotten", async () => {
    const client = new FeatureRowsClient(() => {
      throw new Error("no worker");
    });
    await expect(client.line("a.jsonl", 0)).rejects.toThrow("no worker");
    expect(
      (client as unknown as { pending: Map<number, unknown> }).pending.size,
    ).toBe(0);
  });

  test("retry opens the port again in a new session", () => {
    const fake = fakeWorker();
    const client = new FeatureRowsClient(() => fake.worker);
    client.open("a.jsonl");
    const status = () =>
      fake.worker.onmessage?.(
        statusMessage({ url: "a.jsonl", error: "HTTP 500" }),
      );
    status();
    const before = client.status("a.jsonl")?.session;

    client.retry("a.jsonl");
    expect(fake.sent.at(-1)).toEqual({ type: "open", url: "a.jsonl" });
    status();
    expect(client.status("a.jsonl")?.session).not.toBe(before);
  });

  test("retry and clear keep the search term", async () => {
    const workers: ReturnType<typeof fakeWorker>[] = [];
    const client = new FeatureRowsClient(() => {
      const fake = fakeWorker();
      workers.push(fake);
      return fake.worker;
    });
    client.subscribe("a.jsonl", () => {});
    client.open("a.jsonl");
    workers[0].worker.onmessage?.(
      statusMessage({ url: "a.jsonl", searchTerm: "tokyo", error: "HTTP 500" }),
    );

    client.retry("a.jsonl");
    expect(workers[0].sent.slice(-2)).toEqual([
      { type: "open", url: "a.jsonl" },
      { type: "search", url: "a.jsonl", term: "tokyo" },
    ]);

    workers[0].worker.onmessage?.(
      statusMessage({ url: "a.jsonl", searchTerm: "tokyo" }),
    );
    client.clear();
    await Promise.resolve();
    expect(workers[1].sent).toEqual([
      { type: "open", url: "a.jsonl" },
      { type: "search", url: "a.jsonl", term: "tokyo" },
    ]);
  });
});
