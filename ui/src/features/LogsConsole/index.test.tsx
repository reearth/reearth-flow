import { act, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";

import type { UserFacingLog } from "@flow/types";
import { UserFacingLogLevel } from "@flow/types";

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
  return { response: { text: () => body }, resolve };
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
});
