import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook } from "@testing-library/react";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { useDocument } from "./useApi";

const SaveSnapshot = vi.fn();
vi.mock("../provider", () => ({
  useGraphQLContext: () => ({ SaveSnapshot }),
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
