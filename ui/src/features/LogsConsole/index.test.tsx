import { render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";

import LogsConsole from ".";

const useGetJob = vi.hoisted(() => vi.fn());
const useSubscription = vi.hoisted(() => vi.fn());

vi.mock("@flow/lib/gql/job", () => ({
  useJob: () => ({ useGetJob }),
}));

vi.mock("@flow/lib/gql/subscriptions/useSubscription", () => ({
  useSubscription,
}));

const finishedJob = {
  id: "job-old",
  status: "completed",
  userFacingLogsURL: "https://example.com/job-old/logs.jsonl",
};

const runningJob = { id: "job-new", status: "running" };

describe("LogsConsole", () => {
  beforeEach(() => {
    useGetJob.mockImplementation((jobId: string) => ({
      job: jobId === finishedJob.id ? finishedJob : runningJob,
    }));
    useSubscription.mockReturnValue({ data: undefined });
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue({
        text: () =>
          Promise.resolve(
            JSON.stringify({
              nodeId: "node-1",
              nodeName: "Reader",
              level: "INFO",
              message: "from the previous run",
              timestamp: "2026-10-07T10:00:00Z",
            }),
          ),
      }),
    );
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  // The debug panel keeps this console mounted when a new run replaces a
  // finished one, so logs fetched for the finished run must not carry over.
  test("drops a finished run's logs when a new run starts", async () => {
    const { rerender } = render(<LogsConsole jobId={finishedJob.id} />);
    expect(
      await screen.findByText("from the previous run"),
    ).toBeInTheDocument();

    rerender(<LogsConsole jobId={runningJob.id} />);

    expect(screen.queryByText("from the previous run")).not.toBeInTheDocument();
  });
});
