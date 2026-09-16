import { act, renderHook, waitFor } from "@testing-library/react";
import * as Y from "yjs";

import type { AnyWorkflowVariable, Project, Workspace } from "@flow/types";

import useSharedProjectImport from "./useSharedProjectImport";

type ImportArgs = { workflowVariables: AnyWorkflowVariable[] };

const handleProjectImport = vi.fn(async (_args: ImportArgs) => {});
const navigate = vi.fn();

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

const ALL_VARIABLES = [
  variable("PUBLIC_REGION", true, "ap-northeast-1"),
  variable("PRIVATE_API_KEY", false, "sk-live-do-not-leak"),
  variable("PUBLIC_BUCKET", true, "my-bucket"),
  variable("PRIVATE_DB_PASSWORD", false, "hunter2"),
];

vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => navigate,
}));

vi.mock("@flow/features/NotificationSystem/useToast", () => ({
  useToast: () => ({ toast: vi.fn() }),
}));

vi.mock("@flow/hooks", () => ({
  useProjectImport: () => ({
    isProjectImporting: false,
    handleProjectImport,
  }),
}));

vi.mock("@flow/lib/i18n", () => ({
  useT: () => (key: string) => key,
}));

vi.mock("@flow/lib/gql", () => ({
  // The server hands the shared viewer every variable, values included.
  useWorkflowVariables: () => ({
    useGetWorkflowVariables: () => ({ workflowVariables: ALL_VARIABLES }),
  }),
}));

const runImport = async () => {
  const { result } = renderHook(() =>
    useSharedProjectImport({
      sharedYdoc: new Y.Doc(),
      sharedProject: {
        id: "shared-project",
        name: "Shared",
        description: "",
      } as Project,
      selectedWorkspace: { id: "workspace-1", name: "Target" } as Workspace,
      accessToken: "token",
    }),
  );

  await act(async () => {
    await result.current.handleSharedProjectImport();
  });

  await waitFor(() => expect(handleProjectImport).toHaveBeenCalledTimes(1));
  return handleProjectImport.mock.calls[0][0];
};

describe("useSharedProjectImport", () => {
  beforeEach(() => handleProjectImport.mockClear());

  test("carries every variable over to the imported project", async () => {
    const { workflowVariables } = await runImport();

    expect(workflowVariables.map((v) => v.name)).toEqual([
      "PUBLIC_REGION",
      "PRIVATE_API_KEY",
      "PUBLIC_BUCKET",
      "PRIVATE_DB_PASSWORD",
    ]);
  });

  test("strips private default values but keeps public ones", async () => {
    const { workflowVariables } = await runImport();
    const byName = Object.fromEntries(
      workflowVariables.map((v) => [v.name, v.defaultValue]),
    );

    expect(byName).toEqual({
      PUBLIC_REGION: "ap-northeast-1",
      PUBLIC_BUCKET: "my-bucket",
      PRIVATE_API_KEY: "",
      PRIVATE_DB_PASSWORD: "",
    });
  });

  test("no private value survives anywhere in the import payload", async () => {
    const payload = JSON.stringify(await runImport());

    expect(payload).not.toContain("sk-live-do-not-leak");
    expect(payload).not.toContain("hunter2");
  });
});
