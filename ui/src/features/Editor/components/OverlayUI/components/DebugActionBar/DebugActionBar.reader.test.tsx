import { render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, test, vi } from "vitest";

import {
  EditorProvider,
  type EditorContextType,
} from "@flow/features/Editor/editorContext";
import type { AwarenessUser } from "@flow/types";

import DebugActionBar from "./index";

// A reader may watch and join other people's debug runs, but must never drive
// one. Stopping matters as much as starting here: joining writes the other
// person's job into the local debug state, so an ungated stop button would
// cancel THEIR run.

const state = {
  jobStatus: undefined as string | undefined,
  jobs: [] as { projectId: string; jobId: string; status?: string }[],
};

vi.mock("@flow/stores", async (importOriginal) => ({
  ...(await importOriginal<Record<string, unknown>>()),
  useCurrentProject: () => [{ id: "p1" }, vi.fn()],
}));

vi.mock("@flow/lib/indexedDB", () => ({
  useIndexedDB: () => ({
    value: { jobs: state.jobs },
    updateValue: vi.fn(),
  }),
}));

vi.mock("@flow/lib/gql/subscriptions/useSubscription", () => ({
  useSubscription: () => ({ data: state.jobStatus }),
}));

vi.mock("@flow/lib/gql/job", () => ({
  useJob: () => ({
    useGetJob: () => ({
      job: state.jobs[0]
        ? { id: state.jobs[0].jobId, status: state.jobs[0].status }
        : undefined,
      refetch: vi.fn(),
    }),
  }),
}));

vi.mock("@xyflow/react", async (importOriginal) => ({
  ...(await importOriginal<Record<string, unknown>>()),
  useReactFlow: () => ({ getNodes: () => [] }),
}));

const activeUsersDebugRuns = [
  {
    clientId: 7,
    userName: "Someone Else",
    color: "#fff",
    debugRun: { jobId: "j-other", startedAt: Date.now() },
  },
] as unknown as AwarenessUser[];

const renderBar = (isReaderRestricted: boolean) =>
  render(
    <EditorProvider
      value={
        {
          isLocked: false,
          isReaderRestricted,
          canViewIntermediateData: false,
        } as EditorContextType
      }>
      <DebugActionBar
        activeUsersDebugRuns={activeUsersDebugRuns}
        selectedNodeIds={[]}
        edges={[]}
        isSaving={false}
        onDebugRunJoin={vi.fn()}
        onDebugRunStart={vi.fn()}
        onDebugRunStop={vi.fn()}
        onResetDebugRunWorkflowVariables={vi.fn()}
        refetchWorkflowVariables={vi.fn()}
      />
    </EditorProvider>,
  );

// The bar is all icon buttons with no text, so its controls are addressed by
// position. Popover triggers add role="button" wrapper divs; dropping those
// leaves the real controls in render order:
//   0 start, 1 run-menu caret, 2 stop, 3 clear, 4 join.
const controls = () =>
  screen.getAllByRole("button").filter((el) => el.tagName === "BUTTON");
const startButton = () => controls()[0];
const stopButton = () => controls()[2];
const joinButton = () => controls()[4];

describe("DebugActionBar reader restrictions", () => {
  beforeEach(() => {
    state.jobStatus = undefined;
    state.jobs = [];
  });

  test("a reader cannot start a run, but can still join one", () => {
    renderBar(true);

    expect(startButton()).toBeDisabled();
    // Joining is a read: it stays available so a reader can watch a run.
    expect(joinButton()).toBeEnabled();
  });

  test("a writer can start a run under the same conditions", () => {
    renderBar(false);

    // Same state, only the role differs — so the role is what disables start.
    expect(startButton()).toBeEnabled();
  });

  test("a reader cannot stop a run they joined", () => {
    state.jobStatus = "running";
    state.jobs = [{ projectId: "p1", jobId: "j-other", status: "running" }];

    renderBar(true);

    expect(stopButton()).toBeDisabled();
  });

  test("a writer can stop a running job", () => {
    state.jobStatus = "running";
    state.jobs = [{ projectId: "p1", jobId: "j-mine", status: "running" }];

    renderBar(false);

    expect(stopButton()).toBeEnabled();
  });
});
