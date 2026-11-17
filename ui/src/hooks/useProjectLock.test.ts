import { renderHook } from "@testing-library/react";
import * as Y from "yjs";

import type { Project, YDocMetadataValue } from "@flow/types";

import useProjectLock from "./useProjectLock";

vi.mock("@flow/lib/gql", () => ({
  useProject: () => ({ updateProject: vi.fn() }),
}));

vi.mock("@flow/hooks", () => ({
  useDebouncedCallback: (callback: unknown) => callback,
}));

const makeProject = (isLocked?: boolean): Project => ({
  id: "project-1",
  name: "Project",
  description: "",
  createdAt: "",
  updatedAt: "",
  workspaceId: "workspace-1",
  isLocked,
  alias: "project-alias",
  visibility: "private",
  readme: "",
  license: "",
  topics: [],
  starCount: 0,
  starred: false,
});

const setup = (initialLocked?: boolean) => {
  const yDoc = new Y.Doc();
  const yMetadata = yDoc.getMap<YDocMetadataValue>("metadata");
  if (initialLocked !== undefined) yMetadata.set("isLocked", initialLocked);

  let updates = 0;
  yDoc.on("update", () => updates++);

  return { yDoc, yMetadata, getUpdateCount: () => updates };
};

describe("useProjectLock", () => {
  test("does not write when the doc already matches a locked project", () => {
    const { yDoc, getUpdateCount } = setup(true);

    renderHook(() =>
      useProjectLock({ currentProject: makeProject(true), yDoc }),
    );

    expect(getUpdateCount()).toBe(0);
  });

  test("does not write when the doc already matches an unlocked project", () => {
    const { yDoc, getUpdateCount } = setup(false);

    renderHook(() =>
      useProjectLock({ currentProject: makeProject(false), yDoc }),
    );

    expect(getUpdateCount()).toBe(0);
  });

  test("does not write for an unlocked project whose doc has no key", () => {
    const { yDoc, yMetadata, getUpdateCount } = setup();

    renderHook(() =>
      useProjectLock({ currentProject: makeProject(false), yDoc }),
    );

    expect(getUpdateCount()).toBe(0);
    expect(yMetadata.has("isLocked")).toBe(false);
  });

  test("does not write before the project has loaded", () => {
    const { yDoc, getUpdateCount } = setup(true);

    renderHook(() => useProjectLock({ currentProject: undefined, yDoc }));

    expect(getUpdateCount()).toBe(0);
  });

  test("writes the server value when the doc disagrees", () => {
    const { yDoc, yMetadata, getUpdateCount } = setup(false);

    const { result } = renderHook(() =>
      useProjectLock({ currentProject: makeProject(true), yDoc }),
    );

    expect(getUpdateCount()).toBe(1);
    expect(yMetadata.get("isLocked")).toBe(true);
    expect(result.current.isLocked).toBe(true);
  });
});
