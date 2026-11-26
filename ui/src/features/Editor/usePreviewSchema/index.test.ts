import { act, renderHook } from "@testing-library/react";
import * as Y from "yjs";

import type {
  Job,
  Node,
  NodeSchemaMeta,
  SchemaReport,
  Workflow,
} from "@flow/types";

import usePreviewSchema from ".";

const previewSchema = vi.fn(async () => ({ job: { id: "job-mine" } }));
let projectVariables: unknown[] | undefined = [];

vi.mock("@flow/lib/gql", () => ({
  useProject: () => ({ previewSchema }),
  useWorkflowVariables: () => ({
    useGetWorkflowVariables: () => ({ workflowVariables: projectVariables }),
  }),
}));

vi.mock("@flow/stores", () => ({
  useCurrentProject: () => [
    { id: "project-1", name: "Project", workspaceId: "workspace-1" },
  ],
}));

vi.mock("@flow/utils/toEngineWorkflow/engineReadyWorkflow", () => ({
  createEngineReadyWorkflow: () => ({ id: "engine-workflow" }),
}));

const REPORT_URL =
  "https://example.com/artifacts/job/schema/schema-report.json";

const report: SchemaReport = {
  version: 1,
  sampleSize: 10,
  nodes: {
    reader: {
      name: "Reader",
      ports: {
        default: { open: false, fields: [{ name: "id", type: "String" }] },
      },
    },
  },
};

const readerNode = (schema?: NodeSchemaMeta): Node => ({
  id: "reader",
  type: "reader",
  position: { x: 0, y: 0 },
  data: {
    officialName: "Reader",
    params: { path: "a.csv" },
    nodeMetadata: { schema },
  },
});

const workflows = (schema?: NodeSchemaMeta): Workflow[] => [
  { id: "main", nodes: [readerNode(schema)] },
];

const completedJob = (id: string): Job => ({
  id,
  workspaceId: "workspace-1",
  status: "completed",
  startedAt: "",
  completedAt: "",
  outputURLs: [REPORT_URL],
});

// Persists into a real document, so the tests can watch for updates the way
// the server sees them.
const setup = (schema?: NodeSchemaMeta) => {
  const doc = new Y.Doc();
  const persisted = doc.getMap<Partial<NodeSchemaMeta> | null>("schemas");
  const onPersistSchema = (
    nodeId: string,
    patch: Partial<NodeSchemaMeta> | undefined,
  ) => persisted.set(nodeId, patch ?? null);
  const updates = vi.fn();
  doc.on("update", updates);

  const hook = renderHook(
    ({ rawWorkflows }) => usePreviewSchema({ rawWorkflows, onPersistSchema }),
    { initialProps: { rawWorkflows: workflows(schema) } },
  );
  return { hook, persisted, updates };
};

describe("usePreviewSchema", () => {
  beforeEach(() => {
    previewSchema.mockClear();
    projectVariables = [];
    vi.spyOn(globalThis, "fetch").mockResolvedValue(
      new Response(JSON.stringify(report)),
    );
  });

  afterEach(() => {
    vi.restoreAllMocks();
    vi.useRealTimers();
  });

  test("watches a probe it did not start without writing its outcome", async () => {
    const { hook, updates } = setup({
      ports: {},
      status: "running",
      jobId: "job-other",
    });
    expect(hook.result.current.schemaProbes).toEqual([
      { nodeId: "reader", jobId: "job-other" },
    ]);

    await act(() =>
      hook.result.current.handleProbeComplete(
        "reader",
        completedJob("job-other"),
      ),
    );

    expect(updates).not.toHaveBeenCalled();
    const outcome = hook.result.current.schemaProbeOutcomes["job-other"];
    expect(outcome.status).toBe("complete");
    expect(outcome.ports.default.fields).toEqual([
      { name: "id", type: "String" },
    ]);
    // Worked out, so no longer watched.
    expect(hook.result.current.schemaProbes).toEqual([]);
  });

  test("fails a probe whose job is gone without writing", () => {
    const { hook, updates } = setup({
      ports: {},
      status: "running",
      jobId: "job-orphan",
    });

    act(() => hook.result.current.handleProbeError("reader", "job-orphan"));

    expect(updates).not.toHaveBeenCalled();
    expect(hook.result.current.schemaProbeOutcomes["job-orphan"].status).toBe(
      "failed",
    );
  });

  test("writes the outcome of a probe it started", async () => {
    vi.useFakeTimers();
    const { hook, persisted } = setup();

    act(() => hook.result.current.handleNodeParamsSaved(readerNode()));
    await act(() => vi.runAllTimersAsync());
    expect(previewSchema).toHaveBeenCalledTimes(1);
    expect(persisted.get("reader")).toEqual({
      status: "running",
      jobId: "job-mine",
    });

    hook.rerender({
      rawWorkflows: workflows({
        ports: {},
        status: "running",
        jobId: "job-mine",
      }),
    });
    await act(() =>
      hook.result.current.handleProbeComplete(
        "reader",
        completedJob("job-mine"),
      ),
    );

    expect(persisted.get("reader")?.status).toBe("complete");
    expect(hook.result.current.schemaProbeOutcomes).toEqual({});
  });

  const variableReader = (): Node => ({
    ...readerNode(),
    data: {
      officialName: "Reader",
      params: { dataset: { type: "flowExpr", value: 'variables["url"]' } },
    },
  });
  const urlVariable = (defaultValue: string) => ({
    id: "var-url",
    name: "url",
    defaultValue,
    type: "text",
    required: false,
    public: false,
  });

  test("ignores a probe request that a newer one superseded", async () => {
    vi.useFakeTimers();
    let resolveFirst: (value: { job: { id: string } }) => void = () => {};
    previewSchema
      .mockImplementationOnce(
        () =>
          new Promise((resolve) => {
            resolveFirst = resolve;
          }),
      )
      .mockImplementationOnce(async () => ({ job: { id: "job-second" } }));
    const { hook, persisted } = setup();

    act(() => hook.result.current.handleNodeParamsSaved(readerNode()));
    await act(() => vi.runAllTimersAsync());
    act(() =>
      hook.result.current.handleNodeParamsSaved({
        ...readerNode(),
        data: { ...readerNode().data, params: { path: "b.csv" } },
      }),
    );
    await act(() => vi.runAllTimersAsync());
    expect(persisted.get("reader")).toEqual({
      status: "running",
      jobId: "job-second",
    });

    await act(async () => resolveFirst({ job: { id: "job-first" } }));
    expect(persisted.get("reader")).toEqual({
      status: "running",
      jobId: "job-second",
    });
  });

  test("re-probes a reader when a variable it references changes", async () => {
    vi.useFakeTimers();
    projectVariables = [urlVariable("https://example.com/a.geojson")];
    const { hook } = setup();
    hook.rerender({
      rawWorkflows: [{ id: "main", nodes: [variableReader()] }],
    });

    act(() => hook.result.current.handleNodeParamsSaved(variableReader()));
    await act(() => vi.runAllTimersAsync());
    expect(previewSchema).toHaveBeenCalledTimes(1);

    projectVariables = [urlVariable("https://example.com/b.geojson")];
    hook.rerender({
      rawWorkflows: [{ id: "main", nodes: [variableReader()] }],
    });
    await act(() => vi.runAllTimersAsync());
    expect(previewSchema).toHaveBeenCalledTimes(2);
  });

  test("never probes a reader this client did not save", async () => {
    vi.useFakeTimers();
    projectVariables = [urlVariable("https://example.com/a.geojson")];
    const { hook } = setup();
    projectVariables = [urlVariable("https://example.com/b.geojson")];
    hook.rerender({
      rawWorkflows: [{ id: "main", nodes: [variableReader()] }],
    });
    await act(() => vi.runAllTimersAsync());
    expect(previewSchema).not.toHaveBeenCalled();
  });

  test("waits for the variables to load before probing", async () => {
    vi.useFakeTimers();
    projectVariables = undefined;
    const { hook } = setup();
    hook.rerender({
      rawWorkflows: [{ id: "main", nodes: [variableReader()] }],
    });

    act(() => hook.result.current.handleNodeParamsSaved(variableReader()));
    await act(() => vi.runAllTimersAsync());
    expect(previewSchema).not.toHaveBeenCalled();

    projectVariables = [urlVariable("https://example.com/a.geojson")];
    hook.rerender({
      rawWorkflows: [{ id: "main", nodes: [variableReader()] }],
    });
    await act(() => vi.runAllTimersAsync());
    expect(previewSchema).toHaveBeenCalledTimes(1);
  });
});
