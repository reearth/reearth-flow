import { afterEach, describe, expect, test, vi } from "vitest";

import { putFileWithProgress } from "./putFileWithProgress";

type ProgressInit = {
  loaded: number;
  total: number;
  lengthComputable: boolean;
};

class FakeXHR {
  static last: FakeXHR;

  status = 0;
  statusText = "";
  responseText = "";
  timeout = 0;
  method?: string;
  url?: string;
  sentBody?: unknown;
  headers: Record<string, string> = {};
  aborted = false;

  upload: {
    onprogress?: (e: ProgressInit) => void;
    onload?: () => void;
  } = {};

  onload?: () => void;
  onerror?: () => void;
  ontimeout?: () => void;
  onabort?: () => void;

  constructor() {
    FakeXHR.last = this;
  }

  open(method: string, url: string) {
    this.method = method;
    this.url = url;
  }

  setRequestHeader(name: string, value: string) {
    this.headers[name] = value;
  }

  send(body: unknown) {
    this.sentBody = body;
  }

  abort() {
    this.aborted = true;
    this.onabort?.();
  }

  emitProgress(loaded: number, total: number, lengthComputable = true) {
    this.upload.onprogress?.({ loaded, total, lengthComputable });
  }

  respond(status: number, statusText: string, responseText = "") {
    this.status = status;
    this.statusText = statusText;
    this.responseText = responseText;
    this.upload.onload?.();
    this.onload?.();
  }
}

const install = () => {
  vi.stubGlobal("XMLHttpRequest", FakeXHR);
};

const fakeFile = (size: number) =>
  ({ size, name: "big.geojson" }) as unknown as File;

afterEach(() => {
  vi.unstubAllGlobals();
  vi.useRealTimers();
});

describe("putFileWithProgress", () => {
  test("PUTs the file and resolves with the response", async () => {
    install();
    const promise = putFileWithProgress({
      url: "https://storage.example.com/o",
      file: fakeFile(100),
    });

    const xhr = FakeXHR.last;
    expect(xhr.method).toBe("PUT");
    expect(xhr.url).toBe("https://storage.example.com/o");
    expect(xhr.sentBody).toBe(xhr.sentBody);

    xhr.respond(200, "OK");

    await expect(promise).resolves.toEqual({
      status: 200,
      statusText: "OK",
      responseText: "",
    });
  });

  test("reports progress as it uploads", async () => {
    install();
    const onProgress = vi.fn();
    const promise = putFileWithProgress({
      url: "https://storage.example.com/o",
      file: fakeFile(1000),
      onProgress,
    });

    FakeXHR.last.emitProgress(250, 1000);
    FakeXHR.last.emitProgress(1000, 1000);

    expect(onProgress).toHaveBeenNthCalledWith(1, {
      loaded: 250,
      total: 1000,
      percent: 25,
    });
    expect(onProgress).toHaveBeenNthCalledWith(2, {
      loaded: 1000,
      total: 1000,
      percent: 100,
    });

    FakeXHR.last.respond(200, "OK");
    await promise;
  });

  test("falls back to the file size when the total is not computable", async () => {
    install();
    const onProgress = vi.fn();
    const promise = putFileWithProgress({
      url: "https://storage.example.com/o",
      file: fakeFile(400),
      onProgress,
    });

    FakeXHR.last.emitProgress(100, 0, false);

    expect(onProgress).toHaveBeenCalledWith({
      loaded: 100,
      total: 400,
      percent: 25,
    });

    FakeXHR.last.respond(200, "OK");
    await promise;
  });

  test("resolves non-2xx responses so the caller can read the body", async () => {
    install();
    const promise = putFileWithProgress({
      url: "https://storage.example.com/o",
      file: fakeFile(10),
    });

    FakeXHR.last.respond(403, "Forbidden", "<Error>AccessDenied</Error>");

    await expect(promise).resolves.toEqual({
      status: 403,
      statusText: "Forbidden",
      responseText: "<Error>AccessDenied</Error>",
    });
  });

  test("omits headers with empty values", async () => {
    install();
    const promise = putFileWithProgress({
      url: "https://storage.example.com/o",
      file: fakeFile(10),
      headers: {
        "Content-Type": "",
        "Content-Encoding": null,
        "X-Real": "yes",
      },
    });

    expect(FakeXHR.last.headers).toEqual({ "X-Real": "yes" });

    FakeXHR.last.respond(200, "OK");
    await promise;
  });

  test("rejects when the request never reaches the server", async () => {
    install();
    const promise = putFileWithProgress({
      url: "https://storage.example.com/o",
      file: fakeFile(10),
    });

    FakeXHR.last.onerror?.();

    await expect(promise).rejects.toThrow(/never reached storage/);
  });

  test("rejects once the transfer stops making progress", async () => {
    vi.useFakeTimers();
    install();
    const promise = putFileWithProgress({
      url: "https://storage.example.com/o",
      file: fakeFile(1000),
      stallTimeoutMs: 60_000,
    });

    // Capture the rejection before advancing timers, otherwise it is briefly
    // unhandled and Node warns about it.
    const settled = promise.then(
      () => null,
      (err: Error) => err,
    );

    FakeXHR.last.emitProgress(500, 1000);
    await vi.advanceTimersByTimeAsync(60_000);

    expect(FakeXHR.last.aborted).toBe(true);
    expect((await settled)?.message).toMatch(
      /stalled — no bytes sent for 1 minute/,
    );
  });

  test("does not time out an upload that keeps making progress", async () => {
    vi.useFakeTimers();
    install();
    const promise = putFileWithProgress({
      url: "https://storage.example.com/o",
      file: fakeFile(1000),
      stallTimeoutMs: 60_000,
    });

    // Slow but alive: each chunk re-arms the stall timer.
    for (let loaded = 100; loaded <= 1000; loaded += 100) {
      FakeXHR.last.emitProgress(loaded, 1000);
      await vi.advanceTimersByTimeAsync(50_000);
    }

    expect(FakeXHR.last.aborted).toBe(false);

    FakeXHR.last.respond(200, "OK");
    await expect(promise).resolves.toMatchObject({ status: 200 });
  });

  test("stops the stall timer once the body is fully sent", async () => {
    vi.useFakeTimers();
    install();
    const promise = putFileWithProgress({
      url: "https://storage.example.com/o",
      file: fakeFile(1000),
      stallTimeoutMs: 60_000,
    });

    FakeXHR.last.emitProgress(1000, 1000);
    FakeXHR.last.upload.onload?.();

    // The server may take its time responding; that is not a stalled upload.
    await vi.advanceTimersByTimeAsync(300_000);
    expect(FakeXHR.last.aborted).toBe(false);

    FakeXHR.last.respond(200, "OK");
    await expect(promise).resolves.toMatchObject({ status: 200 });
  });
});
