import { act, renderHook } from "@testing-library/react";
import type { PropsWithChildren } from "react";
import { Awareness } from "y-protocols/awareness";
import * as Y from "yjs";

import { EditorProvider } from "@flow/features/Editor/editorContext";
import type { EditorContextType } from "@flow/features/Editor/editorContext";
import { readWorkflowVarSession } from "@flow/lib/yjs/useWorkflowVarSession";
import type { WorkflowVariable } from "@flow/types";

import useWorkflowVariablesDialog from "./hooks";

vi.mock("@flow/features/NotificationSystem/useToast", () => ({
  useToast: () => ({ toast: vi.fn() }),
}));

vi.mock("@flow/hooks", () => ({
  useWorkflowVars: () => ({
    getUserFacingName: (v: WorkflowVariable) => v.name,
  }),
}));

vi.mock("@flow/lib/i18n", () => ({
  useT: () => (s: string) => s,
}));

const saved: WorkflowVariable[] = [
  {
    id: "v1",
    name: "alpha",
    defaultValue: "",
    type: "text",
    required: false,
    public: true,
  },
];

const setup = () => {
  const yDoc = new Y.Doc();
  const yAwareness = new Awareness(yDoc);
  const docUpdates = vi.fn();
  yDoc.on("update", docUpdates);

  const context: EditorContextType = {
    isLocked: false,
    isReaderRestricted: false,
    canViewIntermediateData: true,
    yDoc,
    yAwareness,
    workflowVarAwareness: {
      onDialogOpen: vi.fn(),
      onDialogClose: vi.fn(),
      onFieldFocus: vi.fn(),
      onEditStart: vi.fn(),
    },
  };
  const wrapper = ({ children }: PropsWithChildren) => (
    <EditorProvider value={context}>{children}</EditorProvider>
  );
  const onClose = vi.fn();
  const onBatchUpdate = vi.fn(async () => {});

  const hook = renderHook(
    () =>
      useWorkflowVariablesDialog({
        currentWorkflowVariables: saved,
        users: {},
        projectId: "project-1",
        onClose,
        onAdd: vi.fn(async () => {}),
        onChange: vi.fn(async () => {}),
        onDelete: vi.fn(async () => {}),
        onBatchUpdate,
      }),
    { wrapper },
  );
  return { hook, yAwareness, docUpdates, onClose, onBatchUpdate };
};

describe("Workflow Variables dialog", () => {
  test("opening and cancelling leave the document untouched", () => {
    const { hook, yAwareness, docUpdates, onClose } = setup();
    expect(hook.result.current.workflowVariables).toEqual(saved);

    act(() => hook.result.current.handleCancel());

    expect(onClose).toHaveBeenCalled();
    expect(docUpdates).not.toHaveBeenCalled();
    expect(readWorkflowVarSession(yAwareness)).toBeUndefined();
  });

  test("editing shares the session through awareness, not the document", () => {
    const { hook, yAwareness, docUpdates } = setup();

    act(() => hook.result.current.handleLocalAdd("text"));

    expect(hook.result.current.workflowVariables).toHaveLength(2);
    expect(hook.result.current.hasUnsavedChanges).toBe(true);
    expect(readWorkflowVarSession(yAwareness)?.base).toEqual(saved);

    act(() => hook.result.current.handleCancel());

    expect(docUpdates).not.toHaveBeenCalled();
    expect(readWorkflowVarSession(yAwareness)).toBeUndefined();
  });

  test("saving goes through the API and leaves the document untouched", async () => {
    const { hook, docUpdates, onBatchUpdate } = setup();

    act(() => hook.result.current.handleUpdate({ ...saved[0], name: "beta" }));
    await act(() => hook.result.current.handleSubmit());

    expect(onBatchUpdate).toHaveBeenCalledWith(
      expect.objectContaining({
        updates: [expect.objectContaining({ paramId: "v1", name: "beta" })],
      }),
    );
    expect(docUpdates).not.toHaveBeenCalled();
  });
});
