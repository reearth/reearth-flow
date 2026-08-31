import { act, cleanup, renderHook } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import * as Y from "yjs";

import type { Edge, Node, Workflow } from "@flow/types";

import { rebuildWorkflow, yWorkflowConstructor } from "./conversions";
import type { YEdgesMap, YNodesMap, YWorkflow } from "./types";
import useYEdge from "./useYEdge";
import useYLayout from "./useYLayout";
import useYNode from "./useYNode";
import useYWorkflow from "./useYWorkflow";

/**
 * A `set` on a Y.Map key beats a concurrent `delete` of that key. So any update
 * that rebuilds a node/edge entry and sets it back will resurrect whatever
 * another client is deleting at that moment — the entry returns while the rest
 * of that delete (its edges, its subworkflow graph) stays gone. These are the
 * ghost nodes and edges.
 *
 * Every update path must therefore mutate the existing entry in place. These
 * tests pin that down for each path that used to rebuild.
 */

afterEach(() => cleanup());

const transformer = (id: string): Node => ({
  id,
  type: "transformer",
  position: { x: 0, y: 0 },
  data: { officialName: id, inputs: ["features"], outputs: ["features"] },
});

const edgeBetween = (id: string, source: string, target: string): Edge => ({
  id,
  source,
  target,
  sourceHandle: "features",
  targetHandle: "features",
});

const twoSyncedDocs = (build: (doc: Y.Doc) => void) => {
  const docA = new Y.Doc();
  const docB = new Y.Doc();
  build(docA);
  Y.applyUpdate(docB, Y.encodeStateAsUpdate(docA));
  // Deliberately left unconnected: both clients edit the same state
  // concurrently, then we merge.
  return {
    docA,
    docB,
    merge: () => {
      Y.applyUpdate(docB, Y.encodeStateAsUpdate(docA), "remote");
      Y.applyUpdate(docA, Y.encodeStateAsUpdate(docB), "remote");
    },
  };
};

const clientFor = (doc: Y.Doc, currentWorkflowId = "main") => {
  const yWorkflows = doc.getMap<YWorkflow>("workflows");
  const rawWorkflows: Workflow[] = Array.from(yWorkflows.entries()).map(
    ([, yw]) => rebuildWorkflow(yw),
  );
  const currentYWorkflow = yWorkflows.get(currentWorkflowId) as YWorkflow;
  const undoTrackerActionWrapper = (cb: () => void) => act(cb);

  const workflow = renderHook(() =>
    useYWorkflow({
      yWorkflows,
      currentWorkflowId,
      rawWorkflows,
      undoTrackerActionWrapper,
    }),
  ).result.current;

  const node = renderHook(() =>
    useYNode({
      currentYWorkflow,
      yWorkflows,
      rawWorkflows,
      setSelectedNodeIds: () => {},
      undoTrackerActionWrapper,
      handleYWorkflowRemove: workflow.handleYWorkflowRemove,
    }),
  ).result.current;

  const edge = renderHook(() =>
    useYEdge({
      currentYWorkflow,
      setSelectedEdgeIds: () => {},
      undoTrackerActionWrapper,
    }),
  ).result.current;

  const layout = renderHook(() =>
    useYLayout({
      currentWorkflowId,
      yWorkflows,
      rawWorkflows,
      undoTrackerActionWrapper,
    }),
  ).result.current;

  return { ...node, ...edge, ...layout };
};

const readState = (doc: Y.Doc, workflowId = "main") => {
  const yWorkflows = doc.getMap<YWorkflow>("workflows");
  const yWorkflow = yWorkflows.get(workflowId) as YWorkflow;
  return {
    graphs: Array.from(yWorkflows.keys()),
    nodes: Object.keys((yWorkflow.get("nodes") as YNodesMap).toJSON()),
    edges: Object.keys((yWorkflow.get("edges") as YEdgesMap).toJSON()),
  };
};

describe("a delete is not undone by a concurrent edit", () => {
  test("reconnecting an edge does not resurrect it after another client deletes it", () => {
    const { docA, docB, merge } = twoSyncedDocs((doc) =>
      doc
        .getMap<YWorkflow>("workflows")
        .set(
          "main",
          yWorkflowConstructor(
            "main",
            "Main",
            [transformer("a"), transformer("b")],
            [edgeBetween("a-b", "a", "b")],
          ),
        ),
    );
    const a = clientFor(docA);
    const b = clientFor(docB);

    // A deletes node b. ReactFlow removes the connected edge first, then the
    // node (see deleteElements in @xyflow/react).
    a.handleYEdgesChange([{ id: "a-b", type: "remove" }]);
    a.handleYNodesChange([{ id: "b", type: "remove" }]);

    // B, at the same time, drags that edge's endpoint onto another port.
    b.handleYEdgesChange([
      {
        id: "a-b",
        type: "replace",
        item: { ...edgeBetween("a-b", "a", "b"), targetHandle: "other" },
      },
    ]);

    merge();

    expect(readState(docA)).toEqual(readState(docB));
    expect(readState(docA).nodes).toEqual(["a"]);
    expect(readState(docA).edges).toEqual([]);
  });

  test("auto-layout does not resurrect nodes another client deleted", () => {
    const { docA, docB, merge } = twoSyncedDocs((doc) =>
      doc
        .getMap<YWorkflow>("workflows")
        .set(
          "main",
          yWorkflowConstructor(
            "main",
            "Main",
            [transformer("a"), transformer("b")],
            [edgeBetween("a-b", "a", "b")],
          ),
        ),
    );
    const a = clientFor(docA);
    const b = clientFor(docB);

    a.handleYEdgesChange([{ id: "a-b", type: "remove" }]);
    a.handleYNodesChange([{ id: "b", type: "remove" }]);

    // Auto-layout rewrites the position of every node in the workflow.
    b.handleYLayoutChange("dagre" as any, "horizontal" as any, false);

    merge();

    expect(readState(docA)).toEqual(readState(docB));
    expect(readState(docA).nodes).toEqual(["a"]);
    expect(readState(docA).edges).toEqual([]);
  });

  test("a subworkflow node is never left behind without its graph", () => {
    // This is the one that kills a run outright: the engine panics on a
    // subGraph node whose graph is missing, which surfaces as an instant
    // failure with no error message.
    const subworkflowNode: Node = {
      id: "sub-1",
      type: "subworkflow",
      position: { x: 0, y: 0 },
      data: { officialName: "Subworkflow", subworkflowId: "sub-1" },
    };

    const { docA, docB, merge } = twoSyncedDocs((doc) => {
      const yWorkflows = doc.getMap<YWorkflow>("workflows");
      yWorkflows.set(
        "main",
        yWorkflowConstructor("main", "Main", [subworkflowNode], []),
      );
      yWorkflows.set("sub-1", yWorkflowConstructor("sub-1", "Sub", [], []));
    });
    const a = clientFor(docA);
    const b = clientFor(docB);

    // Deleting the node also removes its graph - two writes, two keys.
    a.handleYNodesChange([{ id: "sub-1", type: "remove" }]);
    b.handleYLayoutChange("dagre" as any, "horizontal" as any, false);

    merge();

    expect(readState(docA)).toEqual(readState(docB));
    expect(readState(docA).nodes).not.toContain("sub-1");
    expect(readState(docA).graphs).not.toContain("sub-1");
  });

  test("a node moved into a batch is not resurrected by a concurrent delete", () => {
    const { docA, docB, merge } = twoSyncedDocs((doc) =>
      doc
        .getMap<YWorkflow>("workflows")
        .set(
          "main",
          yWorkflowConstructor("main", "Main", [transformer("a")], []),
        ),
    );
    const a = clientFor(docA);
    const b = clientFor(docB);

    a.handleYNodesChange([{ id: "a", type: "remove" }]);
    // "replace" is what dropping a node into a batch emits.
    b.handleYNodesChange([
      {
        id: "a",
        type: "replace",
        item: { ...transformer("a"), parentId: "batch-1" },
      },
    ]);

    merge();

    expect(readState(docA)).toEqual(readState(docB));
    expect(readState(docA).nodes).toEqual([]);
  });
});
