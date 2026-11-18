import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { useQueries } from "./useQueries";

const GetProjectNamedSnapshots = vi.fn();
const GetLatestProjectSnapshot = vi.fn();
const SaveSnapshot = vi.fn();
vi.mock("../provider", () => ({
  useGraphQLContext: () => ({
    GetProjectNamedSnapshots,
    GetLatestProjectSnapshot,
    SaveSnapshot,
  }),
}));

const snapshot = (snapshotNumber: number) => ({
  snapshotNumber,
  label: "auto",
  timestamp: "2026-11-18T00:00:00Z",
  size: 1,
});

const latest = (version: number) => ({
  latestProjectSnapshot: {
    id: "project-1",
    timestamp: "2026-11-18T00:00:00Z",
    updates: [],
    version,
  },
});

// Mirrors the version dialog: it mounts when opened and unmounts when closed.
const openDialog = (queryClient: QueryClient) => {
  const wrapper = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
  return renderHook(
    () => {
      const queries = useQueries();
      return {
        named: queries.useProjectSnapshotsQuery("project-1"),
        latest: queries.useLatestProjectSnapshotQuery("project-1"),
      };
    },
    { wrapper },
  );
};

// The app's default: data stays fresh for 20 seconds.
const appQueryClient = () =>
  new QueryClient({ defaultOptions: { queries: { staleTime: 20 * 1000 } } });

describe("version dialog queries", () => {
  beforeEach(() => {
    GetProjectNamedSnapshots.mockReset();
    GetLatestProjectSnapshot.mockReset();
    SaveSnapshot.mockReset();
  });

  it("fetches versions and the current document again on every open, even while cached data is fresh", async () => {
    const queryClient = appQueryClient();
    GetProjectNamedSnapshots.mockResolvedValueOnce({
      projectNamedSnapshots: [snapshot(1)],
    });
    GetLatestProjectSnapshot.mockResolvedValueOnce(latest(3));

    const first = openDialog(queryClient);
    await waitFor(() =>
      expect(first.result.current.named.snapshots).toHaveLength(1),
    );
    first.unmount();

    GetProjectNamedSnapshots.mockResolvedValueOnce({
      projectNamedSnapshots: [snapshot(2), snapshot(1)],
    });
    GetLatestProjectSnapshot.mockResolvedValueOnce(latest(4));

    const second = openDialog(queryClient);
    await waitFor(() =>
      expect(second.result.current.named.snapshots).toHaveLength(2),
    );
    await waitFor(() =>
      expect(second.result.current.latest.data?.version).toBe(4),
    );
    expect(GetProjectNamedSnapshots).toHaveBeenCalledTimes(2);
    expect(GetLatestProjectSnapshot).toHaveBeenCalledTimes(2);
  });

  it("refreshes an open dialog after a save", async () => {
    const queryClient = appQueryClient();
    GetProjectNamedSnapshots.mockResolvedValueOnce({
      projectNamedSnapshots: [snapshot(1)],
    });
    GetLatestProjectSnapshot.mockResolvedValue(latest(3));

    const dialog = openDialog(queryClient);
    await waitFor(() =>
      expect(dialog.result.current.named.snapshots).toHaveLength(1),
    );

    GetProjectNamedSnapshots.mockResolvedValueOnce({
      projectNamedSnapshots: [snapshot(2), snapshot(1)],
    });
    SaveSnapshot.mockResolvedValueOnce({ saveSnapshot: true });

    const saver = renderHook(() => useQueries(), {
      wrapper: ({ children }: { children: ReactNode }) => (
        <QueryClientProvider client={queryClient}>
          {children}
        </QueryClientProvider>
      ),
    });
    await act(async () => {
      await saver.result.current.snapshotSaveMutation.mutateAsync({
        projectId: "project-1",
      });
    });

    await waitFor(() =>
      expect(dialog.result.current.named.snapshots).toHaveLength(2),
    );
  });
});
