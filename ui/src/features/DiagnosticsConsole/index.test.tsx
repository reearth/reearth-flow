import { render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, test, vi } from "vitest";

import type { Diagnostic, NodeExecution } from "@flow/types";

import DiagnosticsConsole from ".";

const useGetNodeExecutions = vi.hoisted(() => vi.fn());

vi.mock("@flow/lib/gql/job", () => ({
  useJob: () => ({ useGetNodeExecutions }),
}));

const diagnostic = (overrides: Partial<Diagnostic>): Diagnostic => ({
  code: "E000",
  category: "internal",
  severity: "warn",
  message: "message",
  ...overrides,
});

const nodeExecution = (overrides: Partial<NodeExecution>): NodeExecution => ({
  id: "exec-1",
  jobId: "job-1",
  nodeId: "node-1",
  status: "completed",
  ...overrides,
});

describe("DiagnosticsConsole", () => {
  beforeEach(() => {
    useGetNodeExecutions.mockReset();
  });

  const mockNodeExecutions = (nodeExecutions: NodeExecution[]) =>
    useGetNodeExecutions.mockReturnValue({
      nodeExecutions,
      isFetching: false,
    });

  test("flattens diagnostics across every node execution", () => {
    // Diagnostics hang off individual node executions, so a console that only
    // read the first row would silently hide every later node's failures.
    mockNodeExecutions([
      nodeExecution({
        nodeId: "reader",
        diagnostics: [diagnostic({ message: "reader could not open source" })],
      }),
      nodeExecution({
        id: "exec-2",
        nodeId: "writer",
        diagnostics: [diagnostic({ message: "writer skipped a feature" })],
      }),
    ]);

    render(<DiagnosticsConsole jobId="job-1" />);

    expect(
      screen.getByText("reader could not open source"),
    ).toBeInTheDocument();
    expect(screen.getByText("writer skipped a feature")).toBeInTheDocument();
  });

  test("orders diagnostics worst-first", () => {
    // The row a user needs is the fatal one. Engine order is arrival order, so
    // without the sort a fatal row can land below a pile of warnings.
    mockNodeExecutions([
      nodeExecution({
        diagnostics: [
          diagnostic({ severity: "warn", message: "just a warning" }),
          diagnostic({
            severity: "fatal",
            effectiveDisposition: "fatal",
            message: "the actual failure",
          }),
        ],
      }),
    ]);

    render(<DiagnosticsConsole jobId="job-1" />);

    const fatal = screen.getByText("the actual failure");
    const warning = screen.getByText("just a warning");
    expect(
      fatal.compareDocumentPosition(warning) & Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
  });

  test("polls only while the job is active", () => {
    // Neither the jobStatus nor the nodeStatus subscription carries diagnostics,
    // so polling is the only way to see them accumulate during a run — and the
    // only reason to poll a run that has already finished is to waste requests.
    mockNodeExecutions([]);

    const { unmount } = render(
      <DiagnosticsConsole jobId="job-1" isJobActive />,
    );
    expect(useGetNodeExecutions).toHaveBeenLastCalledWith("job-1", true);
    unmount();

    render(<DiagnosticsConsole jobId="job-1" isJobActive={false} />);
    expect(useGetNodeExecutions).toHaveBeenLastCalledWith("job-1", false);
  });

  test("reports an empty result as empty rather than as a failure", () => {
    // Live rows come from a TTL-bound cache that is only merged with the
    // persisted rows at completion, so there is a window right after a run
    // starts where the correct answer is genuinely "nothing yet".
    mockNodeExecutions([]);

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
    mockNodeExecutions([
      nodeExecution({
        diagnostics: [
          diagnostic({
            message: "features dropped",
            aggregatedCount: 1204,
            sampleFeatureIds: ["f1", "f2"],
          }),
        ],
      }),
    ]);

    render(<DiagnosticsConsole jobId="job-1" />);

    expect(screen.getByText("1,204")).toBeInTheDocument();
  });
});
