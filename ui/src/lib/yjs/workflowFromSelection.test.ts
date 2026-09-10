import { act, cleanup, renderHook } from "@testing-library/react";
import { afterEach, describe, expect, test, vi } from "vitest";
import * as Y from "yjs";

import type { Edge, Node } from "@flow/types";

import {
  yEdgeConstructor,
  yNodeConstructor,
  yWorkflowConstructor,
} from "./conversions";
import type { YEdgesMap, YNodesMap, YWorkflow } from "./types";
import useYWorkflow from "./useYWorkflow";

vi.mock("@flow/config", () => ({
  config: () => ({ api: "http://localhost" }),
}));

vi.mock("@flow/lib/i18n", () => ({
  useT: () => (key: string) => key,
}));

// Runs while handleYWorkflowAddFromSelection is awaiting the router configs,
// standing in for whatever the user does during that round trip.
let duringFetch: (() => void) | undefined;

vi.mock("@flow/lib/fetch/transformers/useFetch", () => ({
  fetcher: vi.fn(async (url: string) => {
    duringFetch?.();
    duringFetch = undefined;
    const isInput = url.includes("Input");
    return {
      name: isInput ? "Input Router" : "Output Router",
      type: "transformer",
      inputPorts: isInput ? [] : ["features"],
      outputPorts: isInput ? ["features"] : [],
    };
  }),
}));

afterEach(() => {
  duringFetch = undefined;
  cleanup();
});

const subworkflowNode = (subworkflowId: string): Node => ({
  id: subworkflowId,
  type: "subworkflow",
  position: { x: 0, y: 0 },
  data: { officialName: "Subworkflow", subworkflowId },
});

const transformer = (id: string): Node => ({
  id,
  type: "transformer",
  position: { x: 0, y: 0 },
  data: { officialName: id, inputs: ["features"], outputs: ["features"] },
});

describe("handleYWorkflowAddFromSelection", () => {
  test("classifies edges drawn while the router configs are being fetched", async () => {
    const doc = new Y.Doc();
    const yWorkflows = doc.getMap<YWorkflow>("workflows");
    yWorkflows.set(
      "main",
      yWorkflowConstructor(
        "main",
        "Main",
        [transformer("a"), transformer("b")],
        [],
      ),
    );

    const mainWorkflow = yWorkflows.get("main") as YWorkflow;

    const { result } = renderHook(() =>
      useYWorkflow({
        yWorkflows,
        currentWorkflowId: "main",
        undoTrackerActionWrapper: (cb) => act(cb),
      }),
    );

    // The snapshot the caller passes in: node "a" selected, and no edges yet.
    const snapshotNodes = [
      { ...transformer("a"), selected: true },
      transformer("b"),
    ];
    const snapshotEdges: Edge[] = [];

    // Mid-fetch the user connects the node they are extracting to one they are
    // not, so this edge exists in the document but not in the snapshot.
    duringFetch = () => {
      const yEdges = mainWorkflow.get("edges") as YEdgesMap;
      yEdges.set(
        "a-b",
        yEdgeConstructor({
          id: "a-b",
          source: "a",
          target: "b",
          sourceHandle: "features",
          targetHandle: "features",
        }),
      );
    };

    await act(async () => {
      await result.current.handleYWorkflowAddFromSelection(
        snapshotNodes,
        snapshotEdges,
      );
    });

    const parentNodeIds = Object.keys(
      (mainWorkflow.get("nodes") as YNodesMap).toJSON(),
    );
    const parentEdges = Object.values(
      (mainWorkflow.get("edges") as YEdgesMap).toJSON(),
    ) as Edge[];

    // "a" moved into the new subworkflow...
    expect(parentNodeIds).not.toContain("a");
    // ...so no edge may still be pointing at it.
    expect(
      parentEdges.filter((e) => e.source === "a" || e.target === "a"),
    ).toEqual([]);
    // The connection survives, re-pointed at the subworkflow node.
    expect(parentEdges).toHaveLength(1);
    expect(parentEdges[0].target).toBe("b");
    expect(parentNodeIds).toContain(parentEdges[0].source);
  });

  test("writes nothing when the workflow is replaced during the fetch", async () => {
    const doc = new Y.Doc();
    const yWorkflows = doc.getMap<YWorkflow>("workflows");
    yWorkflows.set(
      "main",
      yWorkflowConstructor(
        "main",
        "Main",
        [transformer("a"), transformer("b")],
        [],
      ),
    );

    const { result } = renderHook(() =>
      useYWorkflow({
        yWorkflows,
        currentWorkflowId: "main",
        undoTrackerActionWrapper: (cb) => act(cb),
      }),
    );

    // The entry itself is swapped out mid-fetch, so the reference captured
    // before the await is now a detached Y.Map that silently swallows writes.
    duringFetch = () => {
      yWorkflows.set(
        "main",
        yWorkflowConstructor(
          "main",
          "Main",
          [transformer("a"), transformer("b")],
          [],
        ),
      );
    };

    await act(async () => {
      await result.current.handleYWorkflowAddFromSelection(
        [{ ...transformer("a"), selected: true }, transformer("b")],
        [],
      );
    });

    // Whatever the handler did, it must have gone into the live entry - never
    // into the detached one, and never half into each.
    const liveWorkflow = yWorkflows.get("main") as YWorkflow;
    const liveNodeIds = Object.keys(
      (liveWorkflow.get("nodes") as YNodesMap).toJSON(),
    );
    const subworkflowIds = Array.from(yWorkflows.keys()).filter(
      (id) => id !== "main",
    );

    // The extraction ran against the entry that is actually in the document:
    // one subworkflow, referenced by a node in the live parent, and the
    // extracted node has left it. Going through the stale reference instead
    // would have created the graph while dropping every write to the parent.
    expect(subworkflowIds).toHaveLength(1);
    expect(liveNodeIds).toContain(subworkflowIds[0]);
    expect(liveNodeIds).not.toContain("a");
  });
});

describe("handleYWorkflowAdd", () => {
  test("derives workflowPath from the document, not a render snapshot", async () => {
    // main (entry) -> sub-1 -> sub-2, except the sub-1 -> sub-2 link is only
    // established while the router configs are being fetched. A path derived
    // from a snapshot taken before the await would come out as "sub-2".
    const doc = new Y.Doc();
    const yWorkflows = doc.getMap<YWorkflow>("workflows");
    yWorkflows.set(
      "main",
      yWorkflowConstructor("main", "Main", [subworkflowNode("sub-1")], []),
    );
    yWorkflows.set("sub-1", yWorkflowConstructor("sub-1", "Sub 1", [], []));
    yWorkflows.set("sub-2", yWorkflowConstructor("sub-2", "Sub 2", [], []));

    const { result } = renderHook(() =>
      useYWorkflow({
        yWorkflows,
        currentWorkflowId: "sub-2",
        undoTrackerActionWrapper: (cb) => act(cb),
      }),
    );

    duringFetch = () => {
      const sub1Nodes = (yWorkflows.get("sub-1") as YWorkflow).get(
        "nodes",
      ) as YNodesMap;
      sub1Nodes.set("sub-2", yNodeConstructor(subworkflowNode("sub-2")));
    };

    await act(async () => {
      await result.current.handleYWorkflowAdd({ x: 0, y: 0 });
    });

    const sub2Nodes = Object.values(
      (
        (yWorkflows.get("sub-2") as YWorkflow).get("nodes") as YNodesMap
      ).toJSON(),
    ) as Node[];
    const created = sub2Nodes.find((n) => n.type === "subworkflow");

    expect(created?.data.workflowPath).toBe("sub-1.sub-2");
  });
});
