import { act, renderHook } from "@testing-library/react";

import type { Project } from "@flow/types";

import useProjectShare from "./useProjectShare";

const shareProject = vi.fn();
const unshareProject = vi.fn();
const refetchProject = vi.fn();

vi.mock("@flow/lib/gql", () => ({
  useSharedProject: () => ({ shareProject, unshareProject }),
  useProject: () => ({ refetchProject }),
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

const urlFor = (token: string) => `${window.location.origin}/shared/${token}`;

const renderShare = (project?: Project) =>
  renderHook(
    ({ currentProject }: { currentProject?: Project }) =>
      useProjectShare({ currentProject }),
    { initialProps: { currentProject: project } },
  );

beforeEach(() => {
  shareProject.mockReset();
  unshareProject.mockReset();
  refetchProject.mockReset();
});

describe("useProjectShare", () => {
  test("takes the sharing URL from the project", () => {
    const { result } = renderShare(makeProject("token-1"));

    expect(result.current.sharingUrl).toBe(urlFor("token-1"));
  });

  test("has no sharing URL for an unshared or unloaded project", () => {
    expect(renderShare(makeProject()).result.current.sharingUrl).toBe(
      undefined,
    );
    expect(renderShare(undefined).result.current.sharingUrl).toBe(undefined);
  });

  test("calls no API on open", () => {
    renderShare(makeProject("token-1"));

    expect(shareProject).not.toHaveBeenCalled();
    expect(unshareProject).not.toHaveBeenCalled();
    expect(refetchProject).not.toHaveBeenCalled();
  });

  test("fetches the project again when the share popover opens", () => {
    const { result } = renderShare(makeProject());

    act(() => result.current.handleProjectShareOpen());

    expect(refetchProject).toHaveBeenCalledWith("project-1");
  });

  test("shows a new share before the project is fetched again", async () => {
    shareProject.mockResolvedValue({
      projectId: "project-1",
      sharingUrl: "https://flow.test/shared/token-2",
    });
    const { result } = renderShare(makeProject());

    await act(() => result.current.handleProjectShare(true));

    expect(shareProject).toHaveBeenCalledWith({
      projectId: "project-1",
      workspaceId: "workspace-1",
    });
    expect(result.current.sharingUrl).toBe(urlFor("token-2"));
  });

  test("keeps the project's state when sharing fails", async () => {
    shareProject.mockResolvedValue({
      projectId: undefined,
      sharingUrl: undefined,
    });
    const { result } = renderShare(makeProject());

    await act(() => result.current.handleProjectShare(true));

    expect(result.current.sharingUrl).toBe(undefined);
  });

  test("hides the URL as soon as unsharing starts", async () => {
    let resolveUnshare: (value: { projectId?: string }) => void = () => {};
    unshareProject.mockReturnValue(
      new Promise((resolve) => (resolveUnshare = resolve)),
    );
    const { result } = renderShare(makeProject("token-1"));

    let pending: Promise<void> = Promise.resolve();
    act(() => {
      pending = result.current.handleProjectShare(false);
    });

    expect(result.current.sharingUrl).toBe(undefined);

    await act(async () => {
      resolveUnshare({ projectId: "project-1" });
      await pending;
    });

    expect(result.current.sharingUrl).toBe(undefined);
  });

  test("restores the URL when unsharing fails", async () => {
    unshareProject.mockResolvedValue({ projectId: undefined });
    const { result } = renderShare(makeProject("token-1"));

    await act(() => result.current.handleProjectShare(false));

    expect(result.current.sharingUrl).toBe(urlFor("token-1"));
  });

  test("follows the project again once it agrees with this user's change", async () => {
    shareProject.mockResolvedValue({
      projectId: "project-1",
      sharingUrl: "https://flow.test/shared/token-2",
    });
    const { result, rerender } = renderShare(makeProject());

    await act(() => result.current.handleProjectShare(true));
    rerender({ currentProject: makeProject("token-2") });
    // Another member unshares later.
    rerender({ currentProject: makeProject() });

    expect(result.current.sharingUrl).toBe(undefined);
  });
});
