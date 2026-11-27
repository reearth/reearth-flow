import { describe, expect, it } from "vitest";
import * as Y from "yjs";

import { DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";
import { yWorkflowConstructor } from "@flow/lib/yjs/conversions";
import type { YWorkflow } from "@flow/lib/yjs/types";
import type { Edge, Node } from "@flow/types";

import { snapshotToWorkflows } from "./snapshotToWorkflows";

const node = (id: string, subworkflowId?: string): Node => ({
  id,
  type: subworkflowId ? "subworkflow" : "transformer",
  position: { x: 0, y: 0 },
  measured: { width: 100, height: 40 },
  dragging: false,
  data: {
    officialName: id,
    inputs: ["default"],
    outputs: ["default"],
    params: { expression: "env.get('x')" },
    ...(subworkflowId ? { subworkflowId } : {}),
  },
});

const edges: Edge[] = [
  {
    id: "e1",
    source: "a",
    target: "b",
    sourceHandle: "default",
    targetHandle: "default",
  },
];

const encodeProject = () => {
  const doc = new Y.Doc();
  const yWorkflows = doc.getMap<YWorkflow>("workflows");
  doc.transact(() => {
    yWorkflows.set(
      DEFAULT_ENTRY_GRAPH_ID,
      yWorkflowConstructor(
        DEFAULT_ENTRY_GRAPH_ID,
        "Main Workflow",
        [node("a"), node("b", "sub-1")],
        edges,
      ),
    );
    yWorkflows.set(
      "sub-1",
      yWorkflowConstructor("sub-1", "Flatten attributes", [node("c")], []),
    );
  });
  return Array.from(Y.encodeStateAsUpdate(doc));
};

describe("snapshotToWorkflows", () => {
  it("rebuilds every workflow, subworkflows included, from snapshot bytes", () => {
    const workflows = snapshotToWorkflows(encodeProject());
    const byId = new Map(workflows.map((w) => [w.id, w]));

    expect(byId.get(DEFAULT_ENTRY_GRAPH_ID)?.name).toBe("Main Workflow");
    expect(byId.get(DEFAULT_ENTRY_GRAPH_ID)?.nodes?.map((n) => n.id)).toEqual([
      "a",
      "b",
    ]);
    expect(byId.get(DEFAULT_ENTRY_GRAPH_ID)?.edges).toEqual(edges);
    expect(byId.get("sub-1")?.nodes?.[0].data.params).toEqual({
      expression: "env.get('x')",
    });
  });

  it("returns no workflows for an empty document", () => {
    expect(
      snapshotToWorkflows(Array.from(Y.encodeStateAsUpdate(new Y.Doc()))),
    ).toEqual([]);
  });
});
