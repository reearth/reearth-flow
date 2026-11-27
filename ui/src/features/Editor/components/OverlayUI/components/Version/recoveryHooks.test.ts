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
    useGetProjectHistory: () => ({
      history: [],
      isFetching: false,
      isError: false,
    }),
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

  // Preview requests are not serialized. Each test below lets the earlier
  // selection's response arrive last, the only order in which it can do harm.
  type Preview = { previewSnapshot?: { updates: number[] }; refused: boolean };
  function deferred() {
    let resolve!: (v: Preview) => void;
    const promise = new Promise<Preview>((r) => (resolve = r));
    return { promise, resolve };
  }
  const selectBoth = (result: ReturnType<typeof render>["result"]) => {
    let earlier: Promise<unknown> = Promise.resolve();
    let latest: Promise<unknown> = Promise.resolve();
    act(() => {
      earlier = result.current.onVersionSelection(2);
    });
    act(() => {
      latest = result.current.onVersionSelection(3);
    });
    return { earlier, latest };
  };

  test("a stale refusal cannot clear the refusal of the version still selected", async () => {
    const first = deferred();
    const second = deferred();
    calls.previewSnapshot
      .mockReturnValueOnce(first.promise)
      .mockReturnValueOnce(second.promise);
    const { result } = render();
    const { earlier, latest } = selectBoth(result);

    await act(async () => {
      second.resolve({ previewSnapshot: undefined, refused: true });
      await latest;
    });
    await act(async () => {
      first.resolve({ previewSnapshot: undefined, refused: true });
      await earlier;
    });

    expect(result.current.selectedProjectSnapshotVersion).toBe(3);
    expect(result.current.isRefusedVersion).toBe(true);
  });

  test("a stale empty preview cannot disable Revert for the version selected", async () => {
    const first = deferred();
    const second = deferred();
    calls.previewSnapshot
      .mockReturnValueOnce(first.promise)
      .mockReturnValueOnce(second.promise);
    const { result } = render();
    const { earlier, latest } = selectBoth(result);

    await act(async () => {
      second.resolve(previewWith(1));
      await latest;
    });
    await act(async () => {
      first.resolve(previewWith(0));
      await earlier;
    });

    expect(result.current.isEmptyPreview).toBe(false);
    expect(result.current.previewDocYWorkflows?.size).toBe(1);
  });

  test("an earlier request finishing does not end the latest one's loading", async () => {
    const first = deferred();
    const second = deferred();
    calls.previewSnapshot
      .mockReturnValueOnce(first.promise)
      .mockReturnValueOnce(second.promise);
    const { result } = render();
    const { earlier, latest } = selectBoth(result);

    await act(async () => {
      first.resolve(previewWith(1));
      await earlier;
    });
    expect(result.current.isLoadingPreview).toBe(true);

    await act(async () => {
      second.resolve(previewWith(1));
      await latest;
    });
    expect(result.current.isLoadingPreview).toBe(false);
  });
});
