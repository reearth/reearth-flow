import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook } from "@testing-library/react";
import type { ReactNode } from "react";
import { describe, expect, it, vi } from "vitest";

import type { ProjectFragment } from "../__gen__/graphql";

import { ProjectQueryKeys, useQueries } from "./useQueries";

const SetProjectStar = vi.fn();
vi.mock("@flow/lib/gql", () => ({
  useGraphQLContext: () => ({ SetProjectStar }),
}));

const fragment = (id: string, starred: boolean): ProjectFragment => ({
  id,
  name: id,
  description: "",
  createdAt: "2026-10-14T00:00:00Z",
  updatedAt: "2026-10-14T00:00:00Z",
  workspaceId: "ws",
  sharedToken: null,
  deployment: null,
  isLocked: false,
  alias: `p-${id}`,
  visibility: "PUBLIC",
  readme: "",
  license: "",
  topics: [],
  starCount: starred ? 1 : 0,
  starred,
});

const setup = () => {
  const queryClient = new QueryClient();
  const pageKey = [ProjectQueryKeys.GetWorkspaceProjects, "ws"];
  queryClient.setQueryData(pageKey, {
    projects: ["a", "b"].map((id) => ({
      id,
      name: id,
      createdAt: "",
      updatedAt: "",
      description: "",
      workspaceId: "ws",
      alias: `p-${id}`,
      visibility: "public",
      readme: "",
      license: "",
      topics: [],
      starCount: 0,
      starred: false,
    })),
    totalCount: 2,
    currentPage: 1,
    totalPages: 1,
  });
  const wrapper = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
  const { result } = renderHook(() => useQueries(), { wrapper });
  const star = (id: string) =>
    queryClient
      .getQueryData<{
        projects: { id: string; starred: boolean; starCount: number }[];
      }>(pageKey)
      ?.projects.find((p) => p.id === id);
  return { result, star };
};

describe("setProjectStarMutation", () => {
  it("rolls back only the project whose star failed", async () => {
    let failA: (err: Error) => void = () => {};
    SetProjectStar.mockImplementation(({ input }) =>
      input.projectId === "a"
        ? new Promise((_, reject) => {
            failA = reject;
          })
        : Promise.resolve({ setProjectStar: { project: fragment("b", true) } }),
    );
    const { result, star } = setup();

    let starA: Promise<unknown> = Promise.resolve();
    await act(async () => {
      starA = result.current.setProjectStarMutation
        .mutateAsync({ projectId: "a", starred: true })
        .catch(() => undefined);
    });
    expect(star("a")).toMatchObject({ starred: true, starCount: 1 });

    await act(() =>
      result.current.setProjectStarMutation.mutateAsync({
        projectId: "b",
        starred: true,
      }),
    );
    expect(star("b")).toMatchObject({ starred: true, starCount: 1 });

    await act(async () => {
      failA(new Error("network"));
      await starA;
    });
    expect(star("a")).toMatchObject({ starred: false, starCount: 0 });
    // B's success survives A's failure.
    expect(star("b")).toMatchObject({ starred: true, starCount: 1 });
  });
});
