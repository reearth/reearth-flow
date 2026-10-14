import { act, renderHook } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { Project } from "@flow/types";

import useProjectForm from "./useProjectForm";

const checkAlias = vi.fn();
let canCreatePrivateProject: boolean | undefined = true;

vi.mock("@flow/lib/gql", () => ({
  useProject: () => ({ useCheckProjectAlias: checkAlias }),
  useWorkspace: () => ({
    useGetWorkspacePolicy: (workspaceId: string) => ({
      policy:
        canCreatePrivateProject === undefined
          ? undefined
          : { workspaceId, canCreatePrivateProject },
    }),
  }),
}));

const project: Project = {
  id: "01abc",
  name: "Roads",
  description: "",
  createdAt: "",
  updatedAt: "",
  workspaceId: "ws",
  alias: "roads",
  visibility: "public",
  readme: "",
  license: "",
  topics: ["gis"],
  starCount: 0,
  starred: false,
};

beforeEach(() => {
  vi.useFakeTimers();
  canCreatePrivateProject = true;
  checkAlias.mockImplementation(() => ({
    availability: undefined,
    errorMessage: undefined,
  }));
});

afterEach(() => {
  vi.useRealTimers();
  checkAlias.mockReset();
});

const setAlias = (
  result: { current: ReturnType<typeof useProjectForm> },
  alias: string,
) => act(() => result.current.setValue("alias", alias));

describe("useProjectForm alias", () => {
  it("lets a new project leave the alias to the server", () => {
    const { result } = renderHook(() => useProjectForm({ workspaceId: "ws" }));
    act(() => result.current.setValue("name", "Roads"));
    expect(result.current.aliasStatus.kind).toBe("default");
    expect(result.current.canSubmit).toBe(true);
    expect(result.current.toCreateInput("ws").alias).toBeUndefined();
  });

  it.each(["abcd", "a".repeat(33), "has space", "dots.dots"])(
    "rejects %j locally",
    (alias) => {
      const { result } = renderHook(() =>
        useProjectForm({ workspaceId: "ws" }),
      );
      setAlias(result, alias);
      expect(result.current.aliasStatus.kind).toBe("invalid");
    },
  );

  it("reserves the p- prefix except for the project's own default", () => {
    const create = renderHook(() => useProjectForm({ workspaceId: "ws" }));
    setAlias(create.result, "p-01abc");
    expect(create.result.current.aliasStatus.kind).toBe("invalid");

    const edit = renderHook(() =>
      useProjectForm({ workspaceId: "ws", project }),
    );
    setAlias(edit.result, "p-other");
    expect(edit.result.current.aliasStatus.kind).toBe("invalid");
    setAlias(edit.result, "p-01abc");
    expect(edit.result.current.aliasStatus.kind).toBe("checking");
  });

  it("asks the server only once typing pauses, passing the project", () => {
    checkAlias.mockImplementation((_ws, alias?: string) => ({
      availability: alias ? { alias, available: alias !== "taken" } : undefined,
      errorMessage: undefined,
    }));
    const { result } = renderHook(() =>
      useProjectForm({ workspaceId: "ws", project }),
    );

    setAlias(result, "new-alias");
    expect(result.current.aliasStatus.kind).toBe("checking");
    expect(checkAlias).toHaveBeenLastCalledWith("ws", undefined, "01abc");

    act(() => vi.advanceTimersByTime(400));
    expect(checkAlias).toHaveBeenLastCalledWith("ws", "new-alias", "01abc");
    expect(result.current.aliasStatus.kind).toBe("available");

    setAlias(result, "taken");
    act(() => vi.advanceTimersByTime(400));
    expect(result.current.aliasStatus.kind).toBe("taken");
    expect(result.current.canSubmit).toBe(false);
  });

  it("does not check or send an unchanged alias", () => {
    const { result } = renderHook(() =>
      useProjectForm({ workspaceId: "ws", project }),
    );
    act(() => vi.advanceTimersByTime(400));
    expect(result.current.aliasStatus.kind).toBe("unchanged");
    expect(checkAlias).toHaveBeenLastCalledWith("ws", undefined, "01abc");
    expect(result.current.toUpdateInput(project).alias).toBeUndefined();
  });
});

describe("useProjectForm visibility", () => {
  it("defaults a new project to public", () => {
    const { result } = renderHook(() => useProjectForm({ workspaceId: "ws" }));
    expect(result.current.values.visibility).toBe("public");
    expect(result.current.toCreateInput("ws").visibility).toBe("PUBLIC");
  });

  it("disables private on a free plan", () => {
    canCreatePrivateProject = false;
    const { result } = renderHook(() =>
      useProjectForm({ workspaceId: "ws", project }),
    );
    expect(result.current.privateDisabled).toBe(true);
    expect(result.current.privateNeedsPaidPlan).toBe(true);

    act(() => result.current.setValue("visibility", "private"));
    expect(result.current.canSubmit).toBe(false);
  });

  it("disables private, without the plan hint, until the policy loads", () => {
    canCreatePrivateProject = undefined;
    const { result } = renderHook(() => useProjectForm({ workspaceId: "ws" }));
    expect(result.current.privateDisabled).toBe(true);
    expect(result.current.privateNeedsPaidPlan).toBe(false);
  });

  it("keeps a private project editable after a downgrade, and warns before making it public", () => {
    canCreatePrivateProject = false;
    const privateProject: Project = { ...project, visibility: "private" };
    const { result } = renderHook(() =>
      useProjectForm({ workspaceId: "ws", project: privateProject }),
    );
    expect(result.current.privateDisabled).toBe(false);
    expect(result.current.canSubmit).toBe(true);
    expect(result.current.losingPrivate).toBe(false);
    expect(
      result.current.toUpdateInput(privateProject).visibility,
    ).toBeUndefined();

    act(() => result.current.setValue("visibility", "public"));
    expect(result.current.losingPrivate).toBe(true);
    expect(result.current.toUpdateInput(privateProject).visibility).toBe(
      "PUBLIC",
    );
  });
});

describe("useProjectForm update input", () => {
  it("sends only the metadata that changed, and [] to clear topics", () => {
    const { result } = renderHook(() =>
      useProjectForm({ workspaceId: "ws", project }),
    );
    expect(result.current.toUpdateInput(project)).toEqual({
      projectId: "01abc",
      name: "Roads",
      description: "",
      alias: undefined,
      visibility: undefined,
      readme: undefined,
      license: undefined,
      topics: undefined,
    });

    act(() => {
      result.current.setValue("topics", []);
      result.current.setValue("readme", "# Roads");
    });
    const input = result.current.toUpdateInput(project);
    expect(input.topics).toEqual([]);
    expect(input.readme).toBe("# Roads");
    expect(input.license).toBeUndefined();
  });
});
