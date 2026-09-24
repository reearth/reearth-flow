import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, test, vi } from "vitest";

import type { Diagnostic } from "@flow/types";

import DiagnosticsConsole from ".";

const useGetJobDiagnostics = vi.hoisted(() => vi.fn());

vi.mock("@flow/lib/gql/job", () => ({
  useJob: () => ({ useGetJobDiagnostics }),
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
    useGetJobDiagnostics.mockReturnValue({
      failedNodes: [],
      bucketRows: [],
      isFetching: false,
    });
  });

  test("shows failedNodes alongside the job-level bucket", () => {
    // The two come from different sources — failedNodes is persisted on the job
    // at completion, the bucket is read live — and a console that rendered only
    // one of them would hide the other entirely.
    useGetJobDiagnostics.mockReturnValue({
      failedNodes: [diagnostic({ message: "the node that failed" })],
      bucketRows: [diagnostic({ message: "a job-level warning" })],
      isFetching: false,
    });

    render(<DiagnosticsConsole jobId="job-1" />);

    expect(screen.getByText("the node that failed")).toBeInTheDocument();
    expect(screen.getByText("a job-level warning")).toBeInTheDocument();
  });

  test("shows a live fatal row while failedNodes is still empty", () => {
    // GetFailedNodes reads only Mongo and the completion artifact, never Redis,
    // so a running job has an empty failedNodes and every fatal row sits in the
    // live bucket. Dropping fatal bucket rows wholesale hid the failure from
    // both the table and the badge until the run ended.
    useGetJobDiagnostics.mockReturnValue({
      failedNodes: [],
      bucketRows: [
        diagnostic({
          severity: "fatal",
          effectiveDisposition: "fatal",
          nodeId: "node-3",
          message: "failing right now",
        }),
      ],
      isFetching: false,
    });

    render(<DiagnosticsConsole jobId="job-1" isJobActive />);

    expect(screen.getByText("failing right now")).toBeInTheDocument();
  });

  test("keeps a live fatal row that failedNodes does not match", () => {
    // Overlap is decided per row, not by disposition: at completion one fatal
    // row can be persisted while another is still only in the live bucket.
    useGetJobDiagnostics.mockReturnValue({
      failedNodes: [
        diagnostic({
          code: "already_persisted",
          severity: "fatal",
          effectiveDisposition: "fatal",
          nodeId: "node-1",
          message: "persisted failure",
        }),
      ],
      bucketRows: [
        diagnostic({
          code: "not_yet_persisted",
          severity: "fatal",
          effectiveDisposition: "fatal",
          nodeId: "node-2",
          message: "live failure",
        }),
      ],
      isFetching: false,
    });

    render(<DiagnosticsConsole jobId="job-1" isJobActive />);

    expect(screen.getByText("persisted failure")).toBeInTheDocument();
    expect(screen.getByText("live failure")).toBeInTheDocument();
  });

  test("renders a fatal row with no nodeId once, not twice", () => {
    // The two sources really do overlap: failedNodes selects on disposition
    // alone and ignores nodeId, so a fatal row carrying no nodeId is returned
    // by both it and the job-level bucket. Rendering the concatenation without
    // filtering showed every such row twice — which is what a failed run
    // actually produces, so this is the common case, not an edge case.
    // Same (nodeId, code, disposition) on both sides: that triple is the
    // server's own dedupe key, and the only thing that makes these one row.
    const terminal = diagnostic({
      code: "internal.unclassified",
      severity: "fatal",
      effectiveDisposition: "fatal",
      nodeId: undefined,
      message: "ExecutionError(Source(...))",
    });
    useGetJobDiagnostics.mockReturnValue({
      failedNodes: [terminal],
      bucketRows: [terminal],
      isFetching: false,
    });

    render(<DiagnosticsConsole jobId="job-1" />);

    expect(screen.getAllByText("ExecutionError(Source(...))")).toHaveLength(1);
  });

  test("keeps a non-fatal job-level row that failedNodes does not carry", () => {
    // The filter drops fatal rows from the bucket, so it must not also swallow
    // the warn/error rows that only the bucket has.
    useGetJobDiagnostics.mockReturnValue({
      failedNodes: [],
      bucketRows: [
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
    useGetJobDiagnostics.mockReturnValue({
      failedNodes: [
        diagnostic({
          severity: "fatal",
          effectiveDisposition: "fatal",
          message: "the actual failure",
        }),
      ],
      bucketRows: [diagnostic({ severity: "warn", message: "just a warning" })],
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

  test("passes every node id through so no bucket is missed", () => {
    // nodeDiagnostics filters on an exact nodeId match and there is no job-wide
    // query, so a node absent from this list contributes nothing and says
    // nothing — the failure that made successful runs look clean.
    render(<DiagnosticsConsole jobId="job-1" nodeIds={["node-7", "node-8"]} />);

    expect(useGetJobDiagnostics).toHaveBeenLastCalledWith("job-1", undefined, [
      "node-7",
      "node-8",
    ]);
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
      failedNodes: [],
      bucketRows: [
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

  test("double-clicking a row reveals the action that reported it", () => {
    // The table names the action but gives no way to reach it; on a large graph
    // hunting down the node by hand is the slow part of reading a failure.
    const onNodeNavigate = vi.fn();
    useGetJobDiagnostics.mockReturnValue({
      failedNodes: [],
      bucketRows: [diagnostic({ nodeId: "node-9", message: "reachable" })],
      isFetching: false,
    });

    render(
      <DiagnosticsConsole jobId="job-1" onNodeNavigate={onNodeNavigate} />,
    );

    fireEvent.doubleClick(screen.getByText("reachable"));

    expect(onNodeNavigate).toHaveBeenCalledWith("node-9");
  });

  test("double-clicking a row with no nodeId navigates nowhere", () => {
    // A failure before the DAG starts carries no node context at all, so there
    // is nothing to fly to — the row must stay inert rather than pick a node.
    const onNodeNavigate = vi.fn();
    useGetJobDiagnostics.mockReturnValue({
      failedNodes: [],
      bucketRows: [diagnostic({ nodeId: undefined, message: "unattributed" })],
      isFetching: false,
    });

    render(
      <DiagnosticsConsole jobId="job-1" onNodeNavigate={onNodeNavigate} />,
    );

    fireEvent.doubleClick(screen.getByText("unattributed"));

    expect(onNodeNavigate).not.toHaveBeenCalled();
  });
});
