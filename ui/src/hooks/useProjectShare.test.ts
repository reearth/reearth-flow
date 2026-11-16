import { renderHook } from "@testing-library/react";
import * as Y from "yjs";

import type { Project, YDocMetadataValue } from "@flow/types";

import useProjectShare from "./useProjectShare";

vi.mock("@flow/lib/gql", () => ({
  useSharedProject: () => ({
    shareProject: vi.fn(),
    unshareProject: vi.fn(),
  }),
}));

const makeProject = (sharedToken?: string): Project => ({
  id: "project-1",
  name: "Project",
  description: "",
  createdAt: "",
  updatedAt: "",
  workspaceId: "workspace-1",
  sharedToken,
  alias: "project-alias",
  visibility: "private",
  readme: "",
  license: "",
  topics: [],
  starCount: 0,
  starred: false,
});

const setup = (initialToken?: string | null) => {
  const yDoc = new Y.Doc();
  const yMetadata = yDoc.getMap<YDocMetadataValue>("metadata");
  if (initialToken !== undefined) yMetadata.set("sharingToken", initialToken);

  let updates = 0;
  yDoc.on("update", () => updates++);

  return { yDoc, yMetadata, getUpdateCount: () => updates };
};

describe("useProjectShare", () => {
  test("does not write when the doc already holds the server token", () => {
    const { yDoc, getUpdateCount } = setup("token-1");

    renderHook(() =>
      useProjectShare({ currentProject: makeProject("token-1"), yDoc }),
    );

    expect(getUpdateCount()).toBe(0);
  });

  test("does not write for an unshared project whose doc has no token", () => {
    const { yDoc, yMetadata, getUpdateCount } = setup();

    renderHook(() =>
      useProjectShare({ currentProject: makeProject(undefined), yDoc }),
    );

    expect(getUpdateCount()).toBe(0);
    expect(yMetadata.has("sharingToken")).toBe(false);
  });

  test("does not write before the project has loaded", () => {
    const { yDoc, getUpdateCount } = setup("token-1");

    renderHook(() => useProjectShare({ currentProject: undefined, yDoc }));

    expect(getUpdateCount()).toBe(0);
  });

  test("writes the server token when the doc disagrees", () => {
    const { yDoc, yMetadata, getUpdateCount } = setup(null);

    const { result } = renderHook(() =>
      useProjectShare({ currentProject: makeProject("token-2"), yDoc }),
    );

    expect(getUpdateCount()).toBe(1);
    expect(yMetadata.get("sharingToken")).toBe("token-2");
    expect(result.current.sharingUrl).toBe(
      `${window.location.origin}/shared/token-2`,
    );
  });

  test("clears the token when the server says the project is unshared", () => {
    const { yDoc, yMetadata } = setup("token-1");

    renderHook(() =>
      useProjectShare({ currentProject: makeProject(undefined), yDoc }),
    );

    expect(yMetadata.get("sharingToken")).toBeNull();
  });
});
