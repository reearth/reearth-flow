import { act, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";

import type { UserFacingLog } from "@flow/types";
import { UserFacingLogLevel } from "@flow/types";

import { logsToJsonl } from "./downloadLogs";

import LogsConsole from ".";

const useGetJob = vi.hoisted(() => vi.fn());
const useSubscription = vi.hoisted(() => vi.fn());

vi.mock("@flow/lib/gql/job", () => ({
  useJob: () => ({ useGetJob }),
}));

vi.mock("@flow/lib/gql/subscriptions/useSubscription", () => ({
  useSubscription,
}));

type Job = { id: string; status: string; userFacingLogsURL?: string };

const finishedJob = (id: string): Job => ({
  id,
  status: "completed",
  userFacingLogsURL: `https://example.com/${id}/logs.jsonl`,
});

const logLine = (message: string) =>
  JSON.stringify({
    nodeId: "node-1",
    nodeName: "Reader",
    level: "INFO",
    message,
    timestamp: "2026-10-07T10:00:00Z",
  });

const liveLog = (jobId: string, message: string): UserFacingLog => ({
  jobId,
  nodeId: "node-1",
  nodeName: "Reader",
  level: UserFacingLogLevel.Info,
  message,
  timestamp: "2026-10-07T10:00:00Z",
});

/** A log file response the test resolves by hand, to order the races. */
const deferredResponse = () => {
  let resolve!: (body: string) => void;
  const body = new Promise<string>((r) => (resolve = r));
  return { response: { ok: true, text: () => body }, resolve };
};

describe("LogsConsole", () => {
  let jobs: Record<string, Job>;
  let liveLogs: Record<string, UserFacingLog[]>;
  let fetchMock: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    jobs = {};
    liveLogs = {};
    useGetJob.mockImplementation((jobId: string) => ({ job: jobs[jobId] }));
    useSubscription.mockImplementation((_key: string, jobId: string) => ({
      data: liveLogs[jobId],
    }));
    fetchMock = vi.fn();
    vi.stubGlobal("fetch", fetchMock);
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  // The debug panel keeps this console mounted when a new run replaces a
  // finished one, so logs fetched for the finished run must not carry over.
  test("drops a finished run's logs when a new run starts", async () => {
    jobs["job-a"] = finishedJob("job-a");
    jobs["job-b"] = { id: "job-b", status: "running" };
    fetchMock.mockResolvedValue({
      ok: true,
      text: () => Promise.resolve(logLine("from the previous run")),
    });

    const { rerender } = render(<LogsConsole jobId="job-a" />);
    expect(
      await screen.findByText("from the previous run"),
    ).toBeInTheDocument();

    rerender(<LogsConsole jobId="job-b" />);

    expect(screen.queryByText("from the previous run")).not.toBeInTheDocument();
  });

  test("shows the new run's live logs while the old run's file is still loading", async () => {
    jobs["job-a"] = finishedJob("job-a");
    jobs["job-b"] = { id: "job-b", status: "running" };
    liveLogs["job-b"] = [liveLog("job-b", "live from the new run")];
    const a = deferredResponse();
    fetchMock.mockResolvedValue(a.response);

    const { rerender } = render(<LogsConsole jobId="job-a" />);
    rerender(<LogsConsole jobId="job-b" />);

    // Not held behind the loading skeleton by job A's request.
    expect(screen.getByText("live from the new run")).toBeInTheDocument();

    await act(async () => a.resolve(logLine("late from the old run")));

    expect(screen.getByText("live from the new run")).toBeInTheDocument();
    expect(screen.queryByText("late from the old run")).not.toBeInTheDocument();
  });

  test("a late response for the old run does not replace the new run's logs", async () => {
    jobs["job-a"] = finishedJob("job-a");
    jobs["job-b"] = finishedJob("job-b");
    const a = deferredResponse();
    const b = deferredResponse();
    fetchMock.mockImplementation((url: string) =>
      Promise.resolve(url.includes("job-a") ? a.response : b.response),
    );

    const { rerender } = render(<LogsConsole jobId="job-a" />);
    rerender(<LogsConsole jobId="job-b" />);

    await act(async () => b.resolve(logLine("from the new run")));
    expect(screen.getByText("from the new run")).toBeInTheDocument();

    await act(async () => a.resolve(logLine("late from the old run")));

    expect(screen.getByText("from the new run")).toBeInTheDocument();
    expect(screen.queryByText("late from the old run")).not.toBeInTheDocument();
    // Nor does it knock job B's result out and send it back for a refetch.
    expect(fetchMock).toHaveBeenCalledTimes(2);
  });

  test("aborts the old run's request when the job changes", () => {
    jobs["job-a"] = finishedJob("job-a");
    jobs["job-b"] = { id: "job-b", status: "running" };
    fetchMock.mockReturnValue(new Promise(() => {}));

    const { rerender } = render(<LogsConsole jobId="job-a" />);
    const { signal } = fetchMock.mock.calls[0][1] as RequestInit;
    expect(signal?.aborted).toBe(false);

    rerender(<LogsConsole jobId="job-b" />);

    expect(signal?.aborted).toBe(true);
  });
  test.each(["failed", "cancelled"])(
    "reads the stored log file for a %s run",
    async (status) => {
      jobs["job-a"] = { ...finishedJob("job-a"), status };
      fetchMock.mockResolvedValue({
        ok: true,
        text: () => Promise.resolve(logLine("from the stored file")),
      });

      render(<LogsConsole jobId="job-a" />);

      expect(
        await screen.findByText("from the stored file"),
      ).toBeInTheDocument();
    },
  );

  test("a finished run with no log lines shows the empty state", async () => {
    jobs["job-a"] = finishedJob("job-a");
    fetchMock.mockResolvedValue({ ok: true, text: () => Promise.resolve("") });

    render(<LogsConsole jobId="job-a" />);

    expect(await screen.findByText("No Logs Available")).toBeInTheDocument();
  });

  test("a missing log file shows the empty state instead of loading", async () => {
    jobs["job-a"] = { ...finishedJob("job-a"), status: "cancelled" };
    fetchMock.mockResolvedValue({
      ok: false,
      text: () => Promise.resolve("<Error>NoSuchKey</Error>"),
    });

    render(<LogsConsole jobId="job-a" />);

    expect(await screen.findByText("No Logs Available")).toBeInTheDocument();
    expect(screen.queryByText(/NoSuchKey/)).not.toBeInTheDocument();
  });

  test("a missing log file keeps the live logs the run still has", async () => {
    jobs["job-a"] = { ...finishedJob("job-a"), status: "cancelled" };
    liveLogs["job-a"] = [liveLog("job-a", "still live")];
    fetchMock.mockResolvedValue({ ok: false, text: () => Promise.resolve("") });

    render(<LogsConsole jobId="job-a" />);

    await act(async () => {});
    expect(screen.getByText("still live")).toBeInTheDocument();
  });

  test("download is off until there are logs to save", () => {
    jobs["job-a"] = { id: "job-a", status: "running" };

    render(<LogsConsole jobId="job-a" />);

    expect(
      screen.getByRole("button", { name: "Download Logs" }),
    ).toBeDisabled();
  });

  test("downloads the logs shown as a JSONL file named for the job", () => {
    jobs["job-a"] = { id: "job-a", status: "running" };
    liveLogs["job-a"] = [liveLog("job-a", "live line")];
    const createObjectURL = vi.fn(() => "blob:logs");
    const revokeObjectURL = vi.fn();
    vi.stubGlobal("URL", { ...URL, createObjectURL, revokeObjectURL });
    const click = vi
      .spyOn(HTMLAnchorElement.prototype, "click")
      .mockImplementation(function (this: HTMLAnchorElement) {
        expect(this.download).toBe("job_job-a_logs.jsonl");
        expect(this.href).toBe("blob:logs");
      });

    render(<LogsConsole jobId="job-a" />);
    fireEvent.click(screen.getByRole("button", { name: "Download Logs" }));

    expect(click).toHaveBeenCalledOnce();
    expect(revokeObjectURL).toHaveBeenCalledWith("blob:logs");
    click.mockRestore();
  });
});

describe("logsToJsonl", () => {
  test("writes one log per line with the stored file's fields", () => {
    const jsonl = logsToJsonl([
      liveLog("job-a", "first"),
      liveLog("job-a", "second"),
    ]);
    const lines = jsonl
      .trimEnd()
      .split("\n")
      .map((l) => JSON.parse(l));

    expect(lines).toEqual([
      {
        timestamp: "2026-10-07T10:00:00Z",
        level: "INFO",
        nodeId: "node-1",
        nodeName: "Reader",
        message: "first",
      },
      {
        timestamp: "2026-10-07T10:00:00Z",
        level: "INFO",
        nodeId: "node-1",
        nodeName: "Reader",
        message: "second",
      },
    ]);
    expect(jsonl.endsWith("\n")).toBe(true);
  });

  test("is empty for no logs", () => {
    expect(logsToJsonl([])).toBe("");
  });
});
