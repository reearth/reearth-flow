import { act, cleanup, renderHook } from "@testing-library/react";
import { describe, test, expect } from "vitest";
import * as Y from "yjs";

import type { Edge } from "@flow/types";

import useYEdge from "./useYEdge";
import { YEdgesArray, YNodesArray, YWorkflow, yWorkflowBuilder } from "./workflowBuilder";

afterEach(() => {
  cleanup();
});

describe("useYEdge", () => {
  test("should update edges correctly", () => {
    const yDoc = new Y.Doc();
    const yWorkflows = yDoc.getArray<YWorkflow>("workflows");
    const yWorkflow = yWorkflowBuilder("main", "Main Workflow");
    yWorkflows.push([yWorkflow]);

    const { result } = renderHook(() => useYEdge(yWorkflow));

    const initialEdges: Edge[] = [
      { id: "1", source: "a", target: "b" },
      { id: "2", source: "b", target: "c" },
    ];
    const newEdges: Edge[] = [{ id: "3", source: "c", target: "d" }];

    const yEdges = yWorkflow.get("edges") as YEdgesArray;
    yEdges.insert(0, initialEdges);

    act(() => {
      result.current.handleEdgesUpdate(newEdges);
    });

    expect(yEdges.toJSON()).toEqual(newEdges);
  });

  test("should not update edges if they are equal", () => {
    const yDoc = new Y.Doc();
    const yWorkflows = yDoc.getArray<YWorkflow>("workflows");
    const yWorkflow = yWorkflowBuilder("main", "Main Workflow");
    yWorkflows.push([yWorkflow]);

    const { result } = renderHook(() => useYEdge(yWorkflow));

    const initialEdges: Edge[] = [{ id: "1", source: "a", target: "b" }];

    const yEdges = yWorkflow.get("edges") as YEdgesArray;
    yEdges.insert(0, initialEdges);

    act(() => {
      result.current.handleEdgesUpdate(initialEdges);
    });

    expect(yEdges.toJSON()).toStrictEqual(initialEdges);
  });

  test("should do nothing if yEdges is undefined", () => {
    const yDoc = new Y.Doc();
    const yWorkflows = yDoc.getArray<YWorkflow>("workflows");
    const yWorkflow = new Y.Map<Y.Text | YNodesArray | YEdgesArray>();
    yWorkflows.push([yWorkflow]);
    const { result } = renderHook(() => useYEdge(yWorkflow));

    const newEdges: Edge[] = [{ id: "1", source: "a", target: "b" }];

    act(() => {
      result.current.handleEdgesUpdate(newEdges);
    });

    expect(yWorkflow.get("edges")).toBeUndefined();
  });
});
