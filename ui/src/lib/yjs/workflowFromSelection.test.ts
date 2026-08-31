import { act, cleanup, renderHook } from "@testing-library/react";
import { afterEach, describe, expect, test, vi } from "vitest";
import * as Y from "yjs";

import type { Edge, Node, Workflow } from "@flow/types";

import {
  rebuildWorkflow,
  yEdgeConstructor,
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
    const rawWorkflows: Workflow[] = [rebuildWorkflow(mainWorkflow)];

    const { result } = renderHook(() =>
      useYWorkflow({
        yWorkflows,
        currentWorkflowId: "main",
        rawWorkflows,
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
});
