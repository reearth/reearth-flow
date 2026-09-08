import { act, renderHook } from "@testing-library/react";

import type { AnyWorkflowVariable, Workspace } from "@flow/types";

import useProjectImport from "./useProjectImport";

type CreateInput = {
  name: string;
  defaultValue: any;
  publicValue: boolean;
};

const createProject = vi.fn(async () => ({ project: { id: "new-project" } }));
const importProject = vi.fn(async () => {});
const updateMultipleWorkflowVariables = vi.fn(
  async (_input: { projectId: string; creates?: CreateInput[] }) => {},
);

vi.mock("@flow/lib/gql", () => ({
  useProject: () => ({ createProject, importProject }),
  useWorkflowVariables: () => ({ updateMultipleWorkflowVariables }),
}));

vi.mock("@flow/lib/i18n", () => ({
  useT: () => (key: string) => key,
}));

const variable = (
  name: string,
  isPublic: boolean,
  defaultValue: any,
): AnyWorkflowVariable =>
  ({
    id: name,
    name,
    defaultValue,
    type: "text",
    required: true,
    public: isPublic,
  }) as AnyWorkflowVariable;

const runImport = async (workflowVariables?: AnyWorkflowVariable[]) => {
  const { result } = renderHook(() => useProjectImport());

  await act(async () => {
    await result.current.handleProjectImport({
      projectName: "Source",
      projectDescription: "",
      workspace: { id: "workspace-1", name: "Target" } as Workspace,
      yDocBinary: new Uint8Array([0, 0]),
      workflowVariables,
    });
  });
};

describe("useProjectImport", () => {
  beforeEach(() => {
    createProject.mockClear();
    importProject.mockClear();
    updateMultipleWorkflowVariables.mockClear();
  });

  test("recreates every variable but strips non-public values", async () => {
    await runImport([
      variable("PUBLIC_REGION", true, "ap-northeast-1"),
      variable("SHOULDNOTAPPEAR", false, "I CANOT be seen"),
      variable("PUBLIC_BUCKET", true, "my-bucket"),
      variable("PRIVATE_PASSWORD", false, "hunter2"),
    ]);

    expect(updateMultipleWorkflowVariables).toHaveBeenCalledTimes(1);
    const { creates } = updateMultipleWorkflowVariables.mock.calls[0][0];

    // Every variable carries over, private ones included.
    expect(creates?.map((c) => c.name)).toEqual([
      "PUBLIC_REGION",
      "SHOULDNOTAPPEAR",
      "PUBLIC_BUCKET",
      "PRIVATE_PASSWORD",
    ]);
    // Public values survive; private ones are blanked.
    expect(
      Object.fromEntries((creates ?? []).map((c) => [c.name, c.defaultValue])),
    ).toEqual({
      PUBLIC_REGION: "ap-northeast-1",
      PUBLIC_BUCKET: "my-bucket",
      SHOULDNOTAPPEAR: "",
      PRIVATE_PASSWORD: "",
    });
  });

  test("no private value reaches the mutation payload", async () => {
    await runImport([
      variable("SHOULDNOTAPPEAR", false, "I CANOT be seen"),
      variable("PRIVATE_PASSWORD", false, "hunter2"),
    ]);

    const payload = JSON.stringify(
      updateMultipleWorkflowVariables.mock.calls[0][0],
    );
    expect(payload).not.toContain("I CANOT be seen");
    expect(payload).not.toContain("hunter2");
  });

  test("keeps the public flag so the importer can tell them apart", async () => {
    await runImport([
      variable("PUBLIC_REGION", true, "ap-northeast-1"),
      variable("PRIVATE_PASSWORD", false, "hunter2"),
    ]);

    const { creates } = updateMultipleWorkflowVariables.mock.calls[0][0];
    expect(creates?.map((c) => c.publicValue)).toEqual([true, false]);
  });

  test("skips the mutation when there are no variables", async () => {
    await runImport([]);
    expect(updateMultipleWorkflowVariables).not.toHaveBeenCalled();
    expect(importProject).toHaveBeenCalled();

    await runImport(undefined);
    expect(updateMultipleWorkflowVariables).not.toHaveBeenCalled();
  });
});
