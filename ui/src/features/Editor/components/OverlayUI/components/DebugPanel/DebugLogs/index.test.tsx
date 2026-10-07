import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, test, vi } from "vitest";

import { EditorProvider } from "@flow/features/Editor/editorContext";
import type { EditorContextType } from "@flow/features/Editor/editorContext";

// vi.mock calls below are hoisted by vitest, so this import still gets the mocks.
import DebugLogs from "./index";

const useJobDiagnostics = vi.hoisted(() => vi.fn());

vi.mock("@flow/hooks/useJobDiagnostics", () => ({
  default: useJobDiagnostics,
}));

vi.mock("@flow/features/LogsConsole", () => ({
  default: ({ leadingActions }: { leadingActions?: React.ReactNode }) => (
    <div>
      the log stream
      {leadingActions}
    </div>
  ),
}));

vi.mock("@flow/features/DiagnosticsConsole", () => ({
  default: ({ leadingActions }: { leadingActions?: React.ReactNode }) => (
    <div>
      the diagnostics table
      {leadingActions}
    </div>
  ),
}));

const renderPanel = (props: { isJobActive?: boolean; debugJobId?: string }) =>
  render(
    <EditorProvider value={{} as EditorContextType}>
      <DebugLogs debugJobId="job-1" {...props} />
    </EditorProvider>,
  );

const diagnostics = (count: number) => {
  useJobDiagnostics.mockReturnValue({ count, hasBlocking: count > 0 });
};

describe("DebugLogs", () => {
  test("switches to the diagnostics once the run has finished", () => {
    // The run is over and it reported something: that is what the user came
    // for, and the view switch that holds it is a single unlabelled icon.
    diagnostics(2);
    const { rerender } = renderPanel({ isJobActive: true });

    expect(screen.getByText("the log stream")).toBeInTheDocument();

    rerender(
      <EditorProvider value={{} as EditorContextType}>
        <DebugLogs debugJobId="job-1" isJobActive={false} />
      </EditorProvider>,
    );

    expect(screen.getByText("the diagnostics table")).toBeInTheDocument();
  });

  test("stays on the logs when a finished run reported nothing", () => {
    diagnostics(0);
    const { rerender } = renderPanel({ isJobActive: true });

    rerender(
      <EditorProvider value={{} as EditorContextType}>
        <DebugLogs debugJobId="job-1" isJobActive={false} />
      </EditorProvider>,
    );

    expect(screen.getByText("the log stream")).toBeInTheDocument();
  });

  test("leaves a run that was already finished on the logs", () => {
    // Reopening the editor must not drop the user into a past run's
    // diagnostics: nothing just happened, so nothing should move.
    diagnostics(3);
    renderPanel({ isJobActive: false });

    expect(screen.getByText("the log stream")).toBeInTheDocument();
  });

  test("does not switch back after the user has picked the logs", () => {
    // Rows keep arriving after the status flips, so the count changes again
    // moments later — and must not pull a reader off the logs a second time.
    diagnostics(1);
    const { rerender } = renderPanel({ isJobActive: true });

    const finished = (
      <EditorProvider value={{} as EditorContextType}>
        <DebugLogs debugJobId="job-1" isJobActive={false} />
      </EditorProvider>
    );
    rerender(finished);
    expect(screen.getByText("the diagnostics table")).toBeInTheDocument();

    // The switch is two icon buttons with tooltips rather than labels; the
    // logs one comes first.
    fireEvent.click(screen.getAllByRole("button")[0]);
    expect(screen.getByText("the log stream")).toBeInTheDocument();

    diagnostics(4);
    rerender(finished);

    expect(screen.getByText("the log stream")).toBeInTheDocument();
  });
});
