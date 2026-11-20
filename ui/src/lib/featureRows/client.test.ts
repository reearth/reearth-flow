import { describe, expect, test } from "vitest";

import { FeatureRowsClient } from "./client";
import type { WorkerLike } from "./client";
import type { FeatureRowsStatus, WorkerRequest } from "./protocol";

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
});
