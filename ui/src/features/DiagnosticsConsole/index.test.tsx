import { render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, test, vi } from "vitest";

import type { Diagnostic } from "@flow/types";

import DiagnosticsConsole from ".";

const useGetJobDiagnostics = vi.hoisted(() => vi.fn());
const useGetJob = vi.hoisted(() => vi.fn());

vi.mock("@flow/lib/gql/job", () => ({
  useJob: () => ({ useGetJob, useGetJobDiagnostics }),
}));

const diagnostic = (overrides: Partial<Diagnostic>): Diagnostic => ({
  code: "E000",
  category: "internal",
  severity: "warn",
  message: "message",
  ...overrides,
});

describe("DiagnosticsConsole", () => {
  beforeEach(() => {
    useGetJobDiagnostics.mockReset();
    useGetJob.mockReset();
    useGetJob.mockReturnValue({ job: undefined });
    useGetJobDiagnostics.mockReturnValue({
      diagnostics: [],
      isFetching: false,
    });
  });

  test("shows failedNodes alongside the job-level bucket", () => {
    // The two come from different sources — failedNodes is persisted on the job
    // at completion, the bucket is read live — and a console that rendered only
    // one of them would hide the other entirely.
    useGetJob.mockReturnValue({
      job: {
        failedNodes: [diagnostic({ message: "the node that failed" })],
      },
    });
    useGetJobDiagnostics.mockReturnValue({
      diagnostics: [diagnostic({ message: "a job-level warning" })],
      isFetching: false,
    });

    render(<DiagnosticsConsole jobId="job-1" />);

    expect(screen.getByText("the node that failed")).toBeInTheDocument();
    expect(screen.getByText("a job-level warning")).toBeInTheDocument();
  });

  test("renders a fatal row with no nodeId once, not twice", () => {
    // The two sources really do overlap: failedNodes selects on disposition
    // alone and ignores nodeId, so a fatal row carrying no nodeId is returned
    // by both it and the job-level bucket. Rendering the concatenation without
    // filtering showed every such row twice — which is what a failed run
    // actually produces, so this is the common case, not an edge case.
    const terminal = diagnostic({
      severity: "fatal",
      effectiveDisposition: "fatal",
      nodeId: undefined,
      message: "ExecutionError(Source(...))",
    });
    useGetJob.mockReturnValue({ job: { failedNodes: [terminal] } });
    useGetJobDiagnostics.mockReturnValue({
      diagnostics: [terminal],
      isFetching: false,
    });

    render(<DiagnosticsConsole jobId="job-1" />);

    expect(screen.getAllByText("ExecutionError(Source(...))")).toHaveLength(1);
  });

  test("keeps a non-fatal job-level row that failedNodes does not carry", () => {
    // The filter drops fatal rows from the bucket, so it must not also swallow
    // the warn/error rows that only the bucket has.
    useGetJob.mockReturnValue({ job: { failedNodes: [] } });
    useGetJobDiagnostics.mockReturnValue({
      diagnostics: [
        diagnostic({ severity: "warn", message: "a job-level warning" }),
      ],
      isFetching: false,
    });

    render(<DiagnosticsConsole jobId="job-1" />);

    expect(screen.getByText("a job-level warning")).toBeInTheDocument();
  });

  test("orders diagnostics worst-first across both sources", () => {
    // The row a user needs is the fatal one. The two sources are concatenated,
    // so without the sort a fatal row can land below a pile of warnings.
    useGetJob.mockReturnValue({
      job: {
        failedNodes: [
          diagnostic({
            severity: "fatal",
            effectiveDisposition: "fatal",
            message: "the actual failure",
          }),
        ],
      },
    });
    useGetJobDiagnostics.mockReturnValue({
      diagnostics: [
        diagnostic({ severity: "warn", message: "just a warning" }),
      ],
      isFetching: false,
    });

    render(<DiagnosticsConsole jobId="job-1" />);

    const fatal = screen.getByText("the actual failure");
    const warning = screen.getByText("just a warning");
    expect(
      fatal.compareDocumentPosition(warning) & Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
  });

  test("polls only while the job is active", () => {
    // Diagnostics never arrive over the jobStatus subscription, so polling is
    // the only way to watch them accumulate during a run — and polling a run
    // that has already finished only wastes requests.
    const { unmount } = render(
      <DiagnosticsConsole jobId="job-1" isJobActive />,
    );
    expect(useGetJobDiagnostics).toHaveBeenLastCalledWith(
      "job-1",
      true,
      undefined,
    );
    unmount();

    render(<DiagnosticsConsole jobId="job-1" isJobActive={false} />);
    expect(useGetJobDiagnostics).toHaveBeenLastCalledWith(
      "job-1",
      false,
      undefined,
    );
  });

  test("asks for a node's own bucket when given a nodeId", () => {
    // nodeDiagnostics filters on an exact nodeId match, so the id has to reach
    // the query verbatim — the default empty bucket is not a superset of it.
    render(<DiagnosticsConsole jobId="job-1" nodeId="node-7" />);

    expect(useGetJobDiagnostics).toHaveBeenLastCalledWith(
      "job-1",
      undefined,
      "node-7",
    );
  });

  test("reports an empty result as empty rather than as a failure", () => {
    // Live rows come from a TTL-bound cache that is only merged with the
    // persisted rows at completion, so there is a window right after a run
    // starts where the correct answer is genuinely "nothing yet".
    render(<DiagnosticsConsole jobId="job-1" isJobActive />);

    expect(
      screen.getByText(
        "No diagnostics reported for this run yet. Diagnostics appear while a run is in progress and are persisted once it finishes.",
      ),
    ).toBeInTheDocument();
  });

  test("surfaces an aggregated row's own count, not a parsed message", () => {
    // aggregatedCount is the structural source for "N features dropped"; the
    // message text is prose and must never be parsed for the number.
    useGetJobDiagnostics.mockReturnValue({
      diagnostics: [
        diagnostic({
          message: "features dropped",
          aggregatedCount: 1204,
          sampleFeatureIds: ["f1", "f2"],
        }),
      ],
      isFetching: false,
    });

    render(<DiagnosticsConsole jobId="job-1" />);

    expect(screen.getByText("1,204")).toBeInTheDocument();
  });
});
