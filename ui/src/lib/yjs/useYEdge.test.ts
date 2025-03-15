import { act, cleanup, renderHook } from "@testing-library/react";
import { describe, test, expect } from "vitest";
import * as Y from "yjs";

import type { Edge } from "@flow/types";

import { yEdgeConstructor, yWorkflowConstructor } from "./conversions";
import { YEdgesMap, YWorkflow } from "./types";
import useYEdge from "./useYEdge";

afterEach(() => {
  cleanup();
});

describe("useYEdge", () => {
  test("should add edges correctly", () => {
    const yDoc = new Y.Doc();
    const yWorkflows = yDoc.getMap<YWorkflow>("workflows");
    const yWorkflow = yWorkflowConstructor("workflow-1", "My Workflow");
    yWorkflows.set("workflow-1", yWorkflow);
    const yEdges = yWorkflow.get("edges") as YEdgesMap;

    const { result } = renderHook(() =>
      useYEdge({
        currentYWorkflow: yWorkflow,
        setSelectedEdgeIds: () => {},
        undoTrackerActionWrapper: (callback) => act(callback),
      }),
    );

    const { handleYEdgesAdd } = result.current;

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

    handleYEdgesAdd(newEdges);

    const expectedArray = Array.from(Object.values(yEdges.toJSON())) as Edge[];

    expect(expectedArray).toEqual(newEdges);
  });

  test("should update edges correctly", () => {
    const yDoc = new Y.Doc();
    const yWorkflows = yDoc.getMap<YWorkflow>("workflows");
    const yWorkflow = yWorkflowConstructor("workflow-1", "My Workflow");
    yWorkflows.set("workflow-1", yWorkflow);

    const initialEdges: Edge[] = [
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

    const initialYEdges = new Y.Map() as YEdgesMap;

    initialEdges.forEach((ie) => {
      const yEdge = yEdgeConstructor(ie);
      initialYEdges.set(ie.id, yEdge);
    });

    yWorkflow.set("edges", initialYEdges);

    const { result } = renderHook(() =>
      useYEdge({
        currentYWorkflow: yWorkflow,
        setSelectedEdgeIds: () => {},
        undoTrackerActionWrapper: (callback) => act(callback),
      }),
    );

    const { handleYEdgesChange } = result.current;

    const newEdges: Edge[] = [
      {
        id: "edge-1",
        source: "node-5",
        target: "node-2",
        sourceHandle: "output1",
        targetHandle: "input1",
      },
      {
        id: "edge-2",
        source: "node-3",
        target: "node-4",
        sourceHandle: "output4",
        targetHandle: "input2",
      },
    ];

    handleYEdgesChange([
      {
        type: "add",
        item: newEdges[0],
      },
      {
        type: "add",
        item: newEdges[1],
      },
    ]);

    const yEdges = yWorkflow.get("edges") as YEdgesMap;

    const expectedArray = Array.from(Object.values(yEdges.toJSON())) as Edge[];

    expect(expectedArray).toEqual(newEdges);
  });
});
