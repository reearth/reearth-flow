import { render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, test, vi } from "vitest";
import * as Y from "yjs";

import {
  EditorProvider,
  type EditorContextType,
} from "@flow/features/Editor/editorContext";

// vi.mock calls below are hoisted by vitest, so this import still gets the mocks.
import VersionDialog from "./index";

const api = {
  getSnapshotState: vi.fn(),
  saveNamedSnapshot: vi.fn(),
  rollbackProject: vi.fn(),
};

const snapshotState = () => {
  const doc = new Y.Doc();
  const workflows = doc.getMap("workflows");
  const workflow = new Y.Map();
  workflow.set("id", new Y.Text("entry"));
  workflow.set("name", new Y.Text("restored"));
  workflow.set("nodes", new Y.Map());
  workflow.set("edges", new Y.Map());
  workflows.set("entry", workflow);
  return Y.encodeStateAsUpdate(doc);
};

vi.mock("@flow/lib/gql/document/useApi", () => ({
  useDocument: () => ({
    useGetProjectNamedSnapshots: () => ({
      snapshots: [
        {
          snapshotNumber: 3,
          label: "before migration",
          timestamp: "2026-07-30T12:00:00Z",
          size: 4096,
        },
        {
          snapshotNumber: 2,
          label: "auto",
          timestamp: "2026-07-30T10:00:00Z",
          size: 3072,
        },
      ],
      isFetching: false,
      isError: false,
    }),
    useGetLatestProjectSnapshot: () => ({
      projectDocument: { version: 41, timestamp: "2026-07-30T13:00:00Z" },
    }),
    useGetProjectNamedSnapshot: api.getSnapshotState,
    useSaveNamedSnapshot: api.saveNamedSnapshot,
    useRollbackProject: api.rollbackProject,
  }),
}));

vi.mock("@flow/features/NotificationSystem/useToast", () => ({
  useToast: () => ({ toast: vi.fn() }),
}));

vi.mock("./components/VersionEditorComponent", () => ({
  // The canvas needs the full editor context; this test is about the panel's
  // interactivity, so it stands in for the preview surface and reports whether a
  // preview document was handed to it.
  default: ({ previewDocYWorkflows }: { previewDocYWorkflows: unknown }) => (
    <div data-testid="preview-surface">
      {previewDocYWorkflows ? "previewing-snapshot" : "showing-live-document"}
    </div>
  ),
}));

describe("VersionDialog interactivity", () => {
  beforeEach(() => {
    Object.values(api).forEach((f) => f.mockReset());
    api.getSnapshotState.mockResolvedValue(snapshotState());
    api.saveNamedSnapshot.mockResolvedValue({ snapshotNumber: 9 });
  });

  const renderDialog = (editorContext: Partial<EditorContextType> = {}) =>
    render(
      <EditorProvider
        value={
          {
            isLocked: false,
            isReaderRestricted: false,
            canViewIntermediateData: false,
            ...editorContext,
          } as EditorContextType
        }>
        <VersionDialog
          project={{ id: "p1" } as never}
          yDoc={new Y.Doc()}
          onDialogClose={() => {}}
        />
      </EditorProvider>,
    );

  test("the panel is interactive, not a read-only list", async () => {
    renderDialog();

    // 1. Rows are real controls, and each shows its own snapshot number.
    const rows = screen.getAllByRole("button", { pressed: false });
    expect(rows.length).toBeGreaterThanOrEqual(2);
    expect(screen.getByText("Snapshot 3")).toBeInTheDocument();
    expect(screen.getByText("Snapshot 2")).toBeInTheDocument();

    // 2. Restore starts unavailable, because nothing is selected yet.
    const restore = screen.getByRole("button", { name: "Restore" });
    expect(restore).toBeDisabled();
    expect(screen.getByTestId("preview-surface")).toHaveTextContent(
      "showing-live-document",
    );

    // 3. Selecting a row previews THAT snapshot.
    screen.getByText("before migration").click();
    await waitFor(() =>
      expect(screen.getByTestId("preview-surface")).toHaveTextContent(
        "previewing-snapshot",
      ),
    );
    expect(api.getSnapshotState).toHaveBeenCalledWith("p1", 3);

    // 4. Restore becomes available and the header follows the selection.
    await waitFor(() => expect(restore).toBeEnabled());
    expect(screen.getByText(/Viewing Snapshot: 3/)).toBeInTheDocument();

    // 5. Restore asks for confirmation rather than acting immediately.
    restore.click();
    await waitFor(() =>
      expect(screen.getByText(/Are you sure/i)).toBeInTheDocument(),
    );

    // Nothing destructive has happened at any point.
    expect(api.rollbackProject).not.toHaveBeenCalled();
  });

  test.each([
    ["the project is locked", { isLocked: true }],
    ["the user is a reader", { isReaderRestricted: true }],
  ])("restore stays unavailable when %s", async (_label, editorContext) => {
    renderDialog(editorContext);

    // Selecting still works — browsing history is a read.
    screen.getByText("before migration").click();
    await waitFor(() =>
      expect(screen.getByTestId("preview-surface")).toHaveTextContent(
        "previewing-snapshot",
      ),
    );

    // But the write is not reachable, so no confirmation can be opened.
    const restore = screen.getByRole("button", { name: "Restore" });
    expect(restore).toBeDisabled();
    restore.click();
    expect(screen.queryByText(/Are you sure/i)).not.toBeInTheDocument();
    expect(api.rollbackProject).not.toHaveBeenCalled();
  });

  test("shows the live document again for an evicted snapshot", async () => {
    // Retention can evict a row between listing and clicking.
    api.getSnapshotState.mockResolvedValue(undefined);
    renderDialog();

    screen.getByText("before migration").click();

    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Restore" })).toBeDisabled(),
    );
    expect(screen.getByTestId("preview-surface")).toHaveTextContent(
      "showing-live-document",
    );
  });
});
