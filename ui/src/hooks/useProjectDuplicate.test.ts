import { act, renderHook } from "@testing-library/react";

import type { Project, Workspace } from "@flow/types";

import useProjectDuplicate from "./useProjectDuplicate";

const createProject = vi.fn(async (_input: { workspaceId: string }) => ({
  project: { id: "new-project" },
}));
const copyProject = vi.fn(
  async (_projectId: string, _source: string, _workspaceId: string) => ({
    success: true,
  }),
);

vi.mock("@flow/lib/gql", () => ({
  useProject: () => ({ createProject, copyProject }),
  useWorkflowVariables: () => ({
    useGetWorkflowVariables: () => ({ workflowVariables: [] }),
    updateMultipleWorkflowVariables: vi.fn(),
  }),
}));

const currentWorkspace = { id: "current", name: "Current" } as Workspace;
vi.mock("@flow/stores", () => ({
  useCurrentWorkspace: () => [currentWorkspace],
}));

const source = { id: "source-project", name: "Source" } as Project;

describe("useProjectDuplicate", () => {
  beforeEach(() => {
    createProject.mockClear();
    copyProject.mockClear();
  });

  test("creates and copies into the chosen workspace", async () => {
    const { result } = renderHook(() => useProjectDuplicate(source));
    const target = { id: "other", name: "Other" } as Workspace;

    let success: boolean | undefined;
    await act(async () => {
      success = await result.current.handleProjectDuplication(source, target);
    });

    expect(success).toBe(true);
    expect(createProject.mock.calls[0][0].workspaceId).toBe("other");
    expect(copyProject).toHaveBeenCalledWith(
      "new-project",
      "source-project",
      "other",
    );
  });

  test("falls back to the current workspace when none is chosen", async () => {
    const { result } = renderHook(() => useProjectDuplicate(source));

    await act(async () => {
      await result.current.handleProjectDuplication(source);
    });

    expect(createProject.mock.calls[0][0].workspaceId).toBe("current");
  });

  test("reports failure when the copy fails", async () => {
    copyProject.mockResolvedValueOnce({ success: false });
    const { result } = renderHook(() => useProjectDuplicate(source));

    let success: boolean | undefined;
    await act(async () => {
      success = await result.current.handleProjectDuplication(source);
    });

    expect(success).toBe(false);
  });
});
