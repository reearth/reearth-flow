import { describe, expect, it } from "vitest";
import * as Y from "yjs";

import { DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";
import { yWorkflowConstructor } from "@flow/lib/yjs/conversions";
import type { YWorkflow } from "@flow/lib/yjs/types";
import type { Edge, Node } from "@flow/types";

import { snapshotToWorkflows } from "./snapshotToWorkflows";

const NODE_A = "6f1c2b9e-1a3d-4c5e-9f70-2b8d4e6a1c01";
const NODE_B = "6f1c2b9e-1a3d-4c5e-9f70-2b8d4e6a1c02";
const NODE_C = "6f1c2b9e-1a3d-4c5e-9f70-2b8d4e6a1c03";
const EDGE_1 = "6f1c2b9e-1a3d-4c5e-9f70-2b8d4e6a1e01";
const SUB_1 = "6f1c2b9e-1a3d-4c5e-9f70-2b8d4e6a1f01";

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
    id: EDGE_1,
    source: NODE_A,
    target: NODE_B,
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
        [node(NODE_A), node(NODE_B, SUB_1)],
        edges,
      ),
    );
    yWorkflows.set(
      SUB_1,
      yWorkflowConstructor(SUB_1, "Flatten attributes", [node(NODE_C)], []),
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
      NODE_A,
      NODE_B,
    ]);
    expect(byId.get(DEFAULT_ENTRY_GRAPH_ID)?.edges).toEqual(edges);
    expect(byId.get(SUB_1)?.nodes?.[0].data.params).toEqual({
      expression: "env.get('x')",
    });
  });

  it("returns no workflows for an empty document", () => {
    expect(
      snapshotToWorkflows(Array.from(Y.encodeStateAsUpdate(new Y.Doc()))),
    ).toEqual([]);
  });
});
