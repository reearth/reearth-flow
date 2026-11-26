import { act, renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, test, vi } from "vitest";
import * as Y from "yjs";

// vi.mock calls below are hoisted by vitest, so this import still gets the mocks.
import useRecoveryHooks from "./recoveryHooks";

const calls = vi.hoisted(() => ({
  previewSnapshot: vi.fn(),
  toast: vi.fn(),
}));

vi.mock("@flow/lib/gql/document/useApi", () => ({
  useDocument: () => ({
    useGetProjectHistory: () => ({ history: [], isFetching: false }),
    useGetLatestProjectSnapshot: () => ({ projectDocument: undefined }),
    useRollbackProject: vi.fn(),
    useGetPreviewProjectSnapshot: calls.previewSnapshot,
  }),
}));

vi.mock("@flow/features/NotificationSystem/useToast", () => ({
  useToast: () => ({ toast: calls.toast }),
}));

vi.mock("@flow/lib/i18n", () => ({
  useT: () => (s: string) => s,
}));

// A preview's state, as an array of numbers the way GraphQL delivers it.
const previewWith = (workflowCount: number) => {
  const doc = new Y.Doc();
  const workflows = doc.getMap("workflows");
  for (let i = 0; i < workflowCount; i++) workflows.set(`w${i}`, new Y.Map());
  return {
    previewSnapshot: { updates: Array.from(Y.encodeStateAsUpdate(doc)) },
    refused: false,
  };
};

const render = () =>
  renderHook(() =>
    useRecoveryHooks({ projectId: "p1", yDoc: null, onDialogClose: () => {} }),
  );

describe("recovery hooks", () => {
  beforeEach(() => {
    calls.previewSnapshot.mockReset();
    calls.toast.mockReset();
  });

  test("marks a refused version and does not report it as a failure", async () => {
    calls.previewSnapshot.mockResolvedValue({
      previewSnapshot: undefined,
      refused: true,
    });
    const { result } = render();

    await act(async () => {
      await result.current.onVersionSelection(2);
    });

    expect(result.current.isRefusedVersion).toBe(true);
    expect(result.current.previewDocYWorkflows).toBeNull();
    expect(calls.toast).not.toHaveBeenCalled();
  });

  test("selecting another version clears the refusal", async () => {
    calls.previewSnapshot
      .mockResolvedValueOnce({ previewSnapshot: undefined, refused: true })
      .mockResolvedValueOnce(previewWith(1));
    const { result } = render();

    await act(async () => {
      await result.current.onVersionSelection(2);
    });
    await act(async () => {
      await result.current.onVersionSelection(3);
    });

    expect(result.current.isRefusedVersion).toBe(false);
    expect(result.current.isEmptyPreview).toBe(false);
  });

  test("marks a preview that has no workflows", async () => {
    calls.previewSnapshot.mockResolvedValue(previewWith(0));
    const { result } = render();

    await act(async () => {
      await result.current.onVersionSelection(2);
    });

    expect(result.current.isEmptyPreview).toBe(true);
    expect(result.current.isRefusedVersion).toBe(false);
  });

  test("leaves an ordinary preview revertable", async () => {
    calls.previewSnapshot.mockResolvedValue(previewWith(2));
    const { result } = render();

    await act(async () => {
      await result.current.onVersionSelection(2);
    });

    expect(result.current.isEmptyPreview).toBe(false);
    expect(result.current.isRefusedVersion).toBe(false);
    expect(result.current.previewDocYWorkflows?.size).toBe(2);
  });
});
