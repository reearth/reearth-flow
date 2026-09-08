import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, test, vi } from "vitest";
import * as Y from "yjs";

// vi.mock calls below are hoisted by vitest, so this import still gets the mocks.
import useHooks from "./hooks";

const calls = {
  getSnapshotState: vi.fn(),
  saveNamedSnapshot: vi.fn(),
  rollbackProject: vi.fn(),
  previewSnapshot: vi.fn(),
};

// A real snapshot state, so revertUpdate operates on decodable bytes.
const snapshotUpdate = (workflowName: string) => {
  const doc = new Y.Doc();
  const workflows = doc.getMap("workflows");
  const workflow = new Y.Map();
  workflow.set("name", new Y.Text(workflowName));
  workflows.set("entry", workflow);
  return Y.encodeStateAsUpdate(doc);
};

vi.mock("@flow/lib/gql/document/useApi", () => ({
  useDocument: () => ({
    useGetProjectNamedSnapshots: () => ({
      snapshots: [
        {
          snapshotNumber: 2,
          label: "before migration",
          timestamp: "",
          size: 1,
        },
      ],
      isFetching: false,
      isError: false,
    }),
    useGetLatestProjectSnapshot: () => ({ projectDocument: undefined }),
    useGetProjectNamedSnapshot: calls.getSnapshotState,
    useSaveNamedSnapshot: calls.saveNamedSnapshot,
    // Present so that a future edit wiring restore to the destructive path is
    // caught by the assertions below rather than passing silently.
    useRollbackProject: calls.rollbackProject,
    useGetPreviewProjectSnapshot: calls.previewSnapshot,
  }),
}));

vi.mock("@flow/features/NotificationSystem/useToast", () => ({
  useToast: () => ({ toast: vi.fn() }),
}));

vi.mock("@flow/lib/i18n", () => ({
  useT: () => (s: string) => s,
}));

describe("Version panel hooks", () => {
  beforeEach(() => {
    Object.values(calls).forEach((c) => c.mockReset());
    calls.getSnapshotState.mockResolvedValue(snapshotUpdate("restored"));
    calls.saveNamedSnapshot.mockResolvedValue({ snapshotNumber: 9 });
  });

  test("selecting a snapshot fetches it by snapshotNumber and builds a preview", async () => {
    const yDoc = new Y.Doc();
    const { result } = renderHook(() =>
      useHooks({ projectId: "p1", yDoc, onDialogClose: () => {} }),
    );

    await act(async () => {
      await result.current.onSnapshotSelect(2);
    });

    // Addressed by snapshotNumber, never by the update-log clock.
    expect(calls.getSnapshotState).toHaveBeenCalledWith("p1", 2);
    await waitFor(() =>
      expect(result.current.previewDocYWorkflows).not.toBeNull(),
    );
    expect(result.current.selectedSnapshotNumber).toBe(2);
  });

  test("restore never calls rollbackProject", async () => {
    const yDoc = new Y.Doc();
    const { result } = renderHook(() =>
      useHooks({ projectId: "p1", yDoc, onDialogClose: () => {} }),
    );

    await act(async () => {
      await result.current.onSnapshotSelect(2);
    });
    await act(async () => {
      await result.current.onSnapshotRestore();
    });

    // rollbackProject reaches PruneAfter, which deletes every update above the
    // number it is given, and snapshotNumber is not that number. This assertion
    // is the guard against reintroducing that data-loss path.
    expect(calls.rollbackProject).not.toHaveBeenCalled();
  });

  test("restore snapshots the current state before reverting", async () => {
    const yDoc = new Y.Doc();
    // Work that exists only in the live doc, as it would between 15m auto-versions.
    yDoc.getMap("workflows").set("scratch", new Y.Map());

    const order: string[] = [];
    calls.saveNamedSnapshot.mockImplementation(async () => {
      order.push("save");
      return { snapshotNumber: 9 };
    });
    yDoc.on("update", () => order.push("revert"));

    const { result } = renderHook(() =>
      useHooks({ projectId: "p1", yDoc, onDialogClose: () => {} }),
    );

    await act(async () => {
      await result.current.onSnapshotSelect(2);
    });
    await act(async () => {
      await result.current.onSnapshotRestore();
    });

    expect(calls.saveNamedSnapshot).toHaveBeenCalledWith(
      "p1",
      "Before restore",
    );
    // Order matters: saving after the revert would snapshot the restored state
    // and leave the replaced work with no entry to return to.
    expect(order[0]).toBe("save");
    expect(order).toContain("revert");
  });

  test("an evicted snapshot does not select or restore", async () => {
    // Retention evicts snapshots, so a listed row can be gone when clicked.
    calls.getSnapshotState.mockResolvedValue(undefined);
    const yDoc = new Y.Doc();
    const { result } = renderHook(() =>
      useHooks({ projectId: "p1", yDoc, onDialogClose: () => {} }),
    );

    await act(async () => {
      await result.current.onSnapshotSelect(2);
    });

    expect(result.current.selectedSnapshotNumber).toBeNull();
    expect(result.current.previewDocYWorkflows).toBeNull();

    await act(async () => {
      await result.current.onSnapshotRestore();
    });
    expect(calls.saveNamedSnapshot).not.toHaveBeenCalled();
  });
});
