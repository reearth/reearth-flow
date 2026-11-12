import { renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, test, vi } from "vitest";
import * as Y from "yjs";

import type { YWorkflow } from "@flow/lib/yjs/types";

import useGraphStaleness from "./useGraphStaleness";

type JobEntry = Record<string, any>;

const state = vi.hoisted(() => ({ value: { jobs: [] as JobEntry[] } }));
const updateValue = vi.hoisted(() =>
  vi.fn((fn: (prev: { jobs: JobEntry[] }) => { jobs: JobEntry[] }) => {
    state.value = fn(state.value);
  }),
);

vi.mock("@flow/lib/indexedDB", () => ({
  useIndexedDB: () => ({ value: state.value, updateValue }),
}));

vi.mock("@flow/stores", () => ({
  useCurrentProject: () => [{ id: "project-1" }],
}));

const buildGraph = () => {
  const doc = new Y.Doc();
  const yWorkflows = doc.getMap<YWorkflow>("workflows");
  doc.transact(() => {
    const yWorkflow = new Y.Map() as YWorkflow;
    const yNodes = new Y.Map<any>();
    const yNode = new Y.Map<any>();
    const yData = new Y.Map<any>();
    yData.set("params", { value: 1 });
    yNode.set("data", yData);
    yNodes.set("node-1", yNode);
    yWorkflow.set("nodes", yNodes);
    yWorkflow.set("edges", new Y.Map<any>());
    yWorkflows.set("workflow-1", yWorkflow);
  });
  const yData = (yWorkflows.get("workflow-1")?.get("nodes") as Y.Map<any>)
    .get("node-1")
    .get("data") as Y.Map<any>;
  return { yWorkflows, yData };
};

describe("useGraphStaleness", () => {
  beforeEach(() => {
    updateValue.mockClear();
  });

  test("marks a node edited during a run as stale", () => {
    const { yWorkflows, yData } = buildGraph();
    state.value = {
      jobs: [{ projectId: "project-1", jobId: "job-1", status: "running" }],
    };

    const { rerender } = renderHook(() =>
      useGraphStaleness({ yWorkflows, undoManager: null }),
    );
    expect(state.value.jobs[0].graphSnapshot).toBeDefined();
    rerender();

    yData.set("params", { value: 2 });

    expect(state.value.jobs[0].isRunStale).toBe(true);
    expect(state.value.jobs[0].staleNodeIds).toEqual(["node-1"]);
  });
});
