import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook } from "@testing-library/react";
import { GraphQLError } from "graphql";
import { ClientError } from "graphql-request";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { useDocument } from "./useApi";

const SaveSnapshot = vi.fn();
const PreviewSnapshot = vi.fn();
const RollbackProject = vi.fn();
vi.mock("../provider", () => ({
  useGraphQLContext: () => ({ SaveSnapshot, PreviewSnapshot, RollbackProject }),
}));

const toast = vi.fn();
vi.mock("@flow/features/NotificationSystem/useToast", () => ({
  useToast: () => ({ toast }),
}));

vi.mock("@flow/lib/i18n", () => ({
  useT: () => (key: string) => key,
}));

const save = async () => {
  const queryClient = new QueryClient({
    defaultOptions: { mutations: { retry: false } },
  });
  const { result } = renderHook(() => useDocument(), {
    wrapper: ({ children }: { children: ReactNode }) => (
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    ),
  });
  let saved: boolean | undefined;
  await act(async () => {
    ({ saveSnapshot: saved } =
      await result.current.useSaveSnapshot("project-1"));
  });
  return saved;
};

describe("useSaveSnapshot", () => {
  beforeEach(() => {
    SaveSnapshot.mockReset();
    toast.mockReset();
  });

  it("reports a new version when the save created one", async () => {
    SaveSnapshot.mockResolvedValueOnce({ saveSnapshot: true });

    expect(await save()).toBe(true);
    expect(toast).toHaveBeenCalledExactlyOnceWith({
      title: "Saved as a new version",
    });
  });

  it("reports nothing to save, without error styling, when nothing changed", async () => {
    SaveSnapshot.mockResolvedValueOnce({ saveSnapshot: false });

    expect(await save()).toBe(false);
    expect(toast).toHaveBeenCalledExactlyOnceWith({
      title: "No changes to save",
      description: "Everything is already in the latest version.",
    });
  });

  it("reports a failure when the save throws", async () => {
    SaveSnapshot.mockRejectedValueOnce(new Error("websocket down"));

    expect(await save()).toBe(false);
    expect(toast).toHaveBeenCalledExactlyOnceWith({
      title: "Project failed to save",
      description: "There was an error saving the project.",
      variant: "destructive",
    });
  });

  it("reports a failure when the save gets no answer", async () => {
    SaveSnapshot.mockResolvedValueOnce(undefined);

    expect(await save()).toBe(false);
    expect(toast).toHaveBeenCalledExactlyOnceWith(
      expect.objectContaining({ variant: "destructive" }),
    );
  });
});

const serverError = (message: string, code?: string) =>
  new ClientError(
    {
      errors: [new GraphQLError(message, code ? { extensions: { code } } : {})],
      status: 200,
      headers: new Headers(),
      body: "",
    },
    { query: "mutation" },
  );

const renderDocument = () => {
  const queryClient = new QueryClient({
    defaultOptions: { mutations: { retry: false } },
  });
  return renderHook(() => useDocument(), {
    wrapper: ({ children }: { children: ReactNode }) => (
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    ),
  }).result;
};

// The recovery view used to receive every failure as "no preview", so a refused
// entry looked empty and stayed revertable. A refusal now arrives as one.
describe("useGetPreviewProjectSnapshot", () => {
  beforeEach(() => PreviewSnapshot.mockReset());

  it("reports an entry the server refuses to rebuild", async () => {
    PreviewSnapshot.mockRejectedValueOnce(
      serverError(
        "this version can no longer be restored",
        "VERSION_NOT_REBUILDABLE",
      ),
    );
    const result = renderDocument();

    let got:
      | Awaited<ReturnType<typeof result.current.useGetPreviewProjectSnapshot>>
      | undefined;
    await act(async () => {
      got = await result.current.useGetPreviewProjectSnapshot("project-1", 3);
    });

    expect(got?.refused).toBe(true);
    expect(got?.previewSnapshot).toBeUndefined();
  });

  it("does not mistake an outage for a refusal", async () => {
    PreviewSnapshot.mockRejectedValueOnce(new Error("network down"));
    const result = renderDocument();

    let got:
      | Awaited<ReturnType<typeof result.current.useGetPreviewProjectSnapshot>>
      | undefined;
    await act(async () => {
      got = await result.current.useGetPreviewProjectSnapshot("project-1", 3);
    });

    expect(got?.refused).toBe(false);
  });
});

describe("useRollbackProject", () => {
  beforeEach(() => {
    RollbackProject.mockReset();
    toast.mockReset();
  });

  it("says a refused version was left unchanged, not that rollback broke", async () => {
    RollbackProject.mockRejectedValueOnce(
      serverError(
        "this version can no longer be restored",
        "VERSION_NOT_REBUILDABLE",
      ),
    );
    const result = renderDocument();

    await act(async () => {
      await result.current.useRollbackProject("project-1", 3);
    });

    expect(toast).toHaveBeenCalledExactlyOnceWith({
      title: "This version can no longer be restored",
      description: "The project was left unchanged.",
      variant: "destructive",
    });
  });

  it("still reports any other failure as a failed rollback", async () => {
    RollbackProject.mockRejectedValueOnce(new Error("network down"));
    const result = renderDocument();

    await act(async () => {
      await result.current.useRollbackProject("project-1", 3);
    });

    expect(toast).toHaveBeenCalledExactlyOnceWith({
      title: "Project Rollback Failed",
      description: "There was an error rolling back the project.",
      variant: "destructive",
    });
  });
});
