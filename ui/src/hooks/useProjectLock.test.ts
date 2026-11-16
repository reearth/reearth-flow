import { act, renderHook, waitFor } from "@testing-library/react";
import * as Y from "yjs";

import type { Project, YDocMetadataValue } from "@flow/types";

import useProjectLock from "./useProjectLock";

const updateProject = vi.fn(
  async (_input: { projectId: string; isLocked: boolean }) => ({
    project: {} as Project | undefined,
  }),
);
const refetchProject = vi.fn();

vi.mock("@flow/lib/gql", () => ({
  useProject: () => ({ updateProject, refetchProject }),
}));

const updatedAt = "2026-11-16T00:00:00.000Z";
const at = Date.parse(updatedAt);

const makeProject = (isLocked: boolean): Project => ({
  id: "project-1",
  name: "Project",
  description: "",
  createdAt: "",
  updatedAt,
  workspaceId: "workspace-1",
  alias: "project-1",
  visibility: "public",
  readme: "",
  license: "",
  topics: [],
  starCount: 0,
  starred: false,
  isLocked,
});

describe("useProjectLock", () => {
  beforeEach(() => {
    updateProject.mockClear();
    refetchProject.mockClear();
  });

  test("reads the lock from the project", () => {
    const yDoc = new Y.Doc();
    const { result, rerender } = renderHook(
      ({ project }) => useProjectLock({ currentProject: project, yDoc }),
      { initialProps: { project: makeProject(false) } },
    );
    expect(result.current.isLocked).toBe(false);

    rerender({ project: makeProject(true) });
    expect(result.current.isLocked).toBe(true);
  });

  test("never writes the lock into the document", async () => {
    const yDoc = new Y.Doc();
    const updates = vi.fn();
    yDoc.on("update", updates);
    const { result } = renderHook(() =>
      useProjectLock({ currentProject: makeProject(false), yDoc }),
    );

    await act(async () => {
      await result.current.handleProjectLockChange(true);
    });

    expect(updateProject).toHaveBeenCalledWith({
      projectId: "project-1",
      isLocked: true,
    });
    expect(updates).not.toHaveBeenCalled();
  });

  test("shows the requested lock until the project has it, and drops it when the update fails", async () => {
    const yDoc = new Y.Doc();
    const { result } = renderHook(() =>
      useProjectLock({ currentProject: makeProject(false), yDoc }),
    );

    let finish: (value: { project: Project | undefined }) => void = () => {};
    updateProject.mockImplementationOnce(
      () => new Promise((resolve) => (finish = resolve)),
    );
    let pending: Promise<void> | undefined;
    act(() => {
      pending = result.current.handleProjectLockChange(true);
    });
    expect(result.current.isLocked).toBe(true);

    await act(async () => {
      finish({ project: undefined });
      await pending;
    });
    expect(result.current.isLocked).toBe(false);
  });

  test("follows a newer lock change from the server and refetches the project", async () => {
    const yDoc = new Y.Doc();
    const { result } = renderHook(() =>
      useProjectLock({ currentProject: makeProject(false), yDoc }),
    );

    act(() => {
      yDoc
        .getMap<YDocMetadataValue>("metadata")
        .set("projectLock", { locked: true, at: at + 1000 });
    });

    expect(result.current.isLocked).toBe(true);
    await waitFor(() =>
      expect(refetchProject).toHaveBeenCalledWith("project-1"),
    );
  });

  test("ignores a lock entry older than the project", () => {
    const yDoc = new Y.Doc();
    yDoc
      .getMap<YDocMetadataValue>("metadata")
      .set("projectLock", { locked: true, at: at - 1000 });

    const { result } = renderHook(() =>
      useProjectLock({ currentProject: makeProject(false), yDoc }),
    );

    expect(result.current.isLocked).toBe(false);
    expect(refetchProject).not.toHaveBeenCalled();
  });
});
