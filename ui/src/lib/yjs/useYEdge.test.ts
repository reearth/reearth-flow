import { act, cleanup, renderHook } from "@testing-library/react";
import { describe, test, expect } from "vitest";
import * as Y from "yjs";

import type { Edge } from "@flow/types";

import { yWorkflowConstructor } from "./conversions";
import { YEdgesArray, YWorkflow } from "./types";
import useYEdge from "./useYEdge";

afterEach(() => {
  cleanup();
});

describe("useYEdge", () => {
  test("should update edges correctly", () => {
    const yDoc = new Y.Doc();
    const yWorkflows = yDoc.getArray<YWorkflow>("workflows");
    const yWorkflow = yWorkflowConstructor("workflow-1", "My Workflow");

    yWorkflows.push([yWorkflow]);

    const { result } = renderHook(() =>
      useYEdge({
        currentYWorkflow: yWorkflow,
        undoTrackerActionWrapper: (callback) => act(callback),
      }),
    );

    const { handleYEdgesUpdate } = result.current;

    const newEdges: Edge[] = [
      {
        id: "edge-1",
        source: "node-1",
        target: "node-2",
        sourceHandle: "output1",
        targetHandle: "input1",
      },
      {
        id: "edge-2",
        source: "node-3",
        target: "node-4",
        sourceHandle: "output2",
        targetHandle: "input2",
      },
    ];

    handleYEdgesUpdate(newEdges);

    const yEdges = yWorkflow.get("edges") as YEdgesArray;

    const e = yEdges.toJSON() as Edge[];

    expect(e).toEqual(newEdges);
  });
});
