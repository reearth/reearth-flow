import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, test, vi } from "vitest";

import { EditorProvider } from "@flow/features/Editor/editorContext";
import type { EditorContextType } from "@flow/features/Editor/editorContext";
import type { Node } from "@flow/types";

import DebugActionBar from "./index";

const state = {
  jobStatus: undefined as string | undefined,
  jobs: [] as { projectId: string; jobId: string; status?: string }[],
  nodes: [] as Node[],
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
    useGetJob: () => ({ job: undefined, refetch: vi.fn() }),
  }),
}));

vi.mock("@xyflow/react", async (importOriginal) => ({
  ...(await importOriginal<Record<string, unknown>>()),
  useReactFlow: () => ({ getNodes: () => state.nodes }),
}));

const action = (id: string, type: Node["type"], isDisabled = false): Node => ({
  id,
  type,
  position: { x: 0, y: 0 },
  data: { officialName: id, isDisabled },
});

const renderBar = ({
  selectedNodeIds,
  canDebugRunFromNode,
}: {
  selectedNodeIds: string[];
  canDebugRunFromNode?: (nodeId: string) => boolean;
}) =>
  render(
    <EditorProvider
      value={
        {
          isLocked: false,
          isReaderRestricted: false,
          canViewIntermediateData: false,
          canDebugRunFromNode,
        } as EditorContextType
      }>
      <DebugActionBar
        selectedNodeIds={selectedNodeIds}
        edges={[{ id: "e", source: "R", target: "A" }]}
        isSaving={false}
        onDebugRunStart={vi.fn()}
        onDebugRunStartFromSelectedNode={vi.fn()}
        onDebugRunStartToSelectedNode={vi.fn()}
        onDebugRunStop={vi.fn()}
        onResetDebugRunWorkflowVariables={vi.fn()}
        refetchWorkflowVariables={vi.fn()}
      />
    </EditorProvider>,
  );

const openMenu = () => {
  const caret = screen
    .getAllByRole("button")
    .filter((el) => el.tagName === "BUTTON")[1];
  fireEvent.click(caret);
};

const menuItem = (label: string) =>
  screen.getByText(label).closest('[role="menuitem"]');

describe("Run To Selected Action", () => {
  beforeEach(() => {
    state.jobStatus = undefined;
    state.jobs = [];
    state.nodes = [
      action("R", "reader"),
      action("A", "transformer"),
      action("W", "writer"),
      action("N", "note"),
      action("D", "transformer", true),
    ];
  });

  test("is available without any earlier run", () => {
    renderBar({ selectedNodeIds: ["A"] });
    openMenu();

    expect(menuItem("Run To Selected Action")).not.toHaveAttribute(
      "aria-disabled",
      "true",
    );
  });

  test("is available for a writer", () => {
    renderBar({ selectedNodeIds: ["W"] });
    openMenu();

    expect(menuItem("Run To Selected Action")).not.toHaveAttribute(
      "aria-disabled",
      "true",
    );
  });

  test("is unavailable for a note, a disabled action or several actions", () => {
    for (const selectedNodeIds of [["N"], ["D"], ["R", "A"], []]) {
      const { unmount } = renderBar({ selectedNodeIds });
      openMenu();
      expect(menuItem("Run To Selected Action")).toHaveAttribute(
        "aria-disabled",
        "true",
      );
      unmount();
    }
  });

  test("is unavailable while a run is going", () => {
    state.jobStatus = "running";
    state.jobs = [{ projectId: "p1", jobId: "j1", status: "running" }];
    renderBar({ selectedNodeIds: ["A"] });
    openMenu();

    expect(menuItem("Run To Selected Action")).toHaveAttribute(
      "aria-disabled",
      "true",
    );
  });
});

describe("Run From Selected Action after a partial run", () => {
  beforeEach(() => {
    state.jobStatus = "completed";
    state.jobs = [{ projectId: "p1", jobId: "j1", status: "completed" }];
    state.nodes = [action("R", "reader"), action("A", "transformer")];
  });

  test("is available when the last run covers what it would reuse", () => {
    renderBar({ selectedNodeIds: ["A"], canDebugRunFromNode: () => true });
    openMenu();

    expect(menuItem("Run From Selected Action")).not.toHaveAttribute(
      "aria-disabled",
      "true",
    );
  });

  test("is unavailable when the last run skipped something it would reuse", () => {
    renderBar({ selectedNodeIds: ["A"], canDebugRunFromNode: () => false });
    openMenu();

    expect(menuItem("Run From Selected Action")).toHaveAttribute(
      "aria-disabled",
      "true",
    );
  });
});
