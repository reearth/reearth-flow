// import { act, cleanup, renderHook } from "@testing-library/react";
// import { describe, test, expect } from "vitest";
// import * as Y from "yjs";

import { cleanup } from "@testing-library/react";

// import type { Node } from "@flow/types";

// import useYNode from "./useYNode";
// import {
//   YEdgesArray,
//   YNodesArray,
//   YWorkflow,
//   yWorkflowConstructor,
// } from "./workflowBuilder";

afterEach(() => {
  cleanup();
});

describe("useYNode", () => {
  test("should update nodes correctly", () => {
    // const yDoc = new Y.Doc();
    // const yWorkflows = yDoc.getArray<YWorkflow>("workflows");
    // const yWorkflow = yWorkflowConstructor("main", "Main Workflow");
    // yWorkflows.push([yWorkflow]);
    // const { result } = renderHook(() =>
    //   useYNode({
    //     currentYWorkflow: yWorkflow,
    //     handleWorkflowsRemove: () => {},
    //     undoTrackerActionWrapper: () => {},
    //   }),
    // );
    // const initialNodes: Node[] = [
    //   { id: "a", position: { x: 0, y: 0 }, data: { name: "Node A" } },
    //   { id: "b", position: { x: 0, y: 0 }, data: { name: "Node B" } },
    // ];
    // const newNodes: Node[] = [
    //   { id: "c", position: { x: 0, y: 0 }, data: { name: "Node C" } },
    // ];
    // const yNodes = yWorkflow.get("nodes") as YNodesArray;
    // yNodes.insert(0, initialNodes);
    // act(() => {
    //   result.current.handleNodesUpdate(newNodes);
    // });
    // expect(yNodes.toJSON()).toEqual(newNodes);
  });

  test("should not update edges if they are equal", () => {
    // const yDoc = new Y.Doc();
    // const yWorkflows = yDoc.getArray<YWorkflow>("workflows");
    // const yWorkflow = yWorkflowConstructor("main", "Main Workflow");
    // yWorkflows.push([yWorkflow]);
    // const { result } = renderHook(() =>
    //   useYNode({
    //     currentYWorkflow: yWorkflow,
    //     handleWorkflowsRemove: () => {},
    //     undoTrackerActionWrapper: () => {},
    //   }),
    // );
    // const initialNodes: Node[] = [
    //   { id: "a", position: { x: 0, y: 0 }, data: { name: "Node A" } },
    //   { id: "b", position: { x: 0, y: 0 }, data: { name: "Node B" } },
    // ];
    // const yNodes = yWorkflow.get("nodes") as YNodesArray;
    // yNodes.insert(0, initialNodes);
    // act(() => {
    //   result.current.handleNodesUpdate(initialNodes);
    // });
    // expect(yNodes.toJSON()).toStrictEqual(initialNodes);
  });

  test("should do nothing if yEdges is undefined", () => {
    // const yDoc = new Y.Doc();
    // const yWorkflows = yDoc.getArray<YWorkflow>("workflows");
    // const yWorkflow = new Y.Map<Y.Text | YNodesArray | YEdgesArray>();
    // yWorkflows.push([yWorkflow]);
    // const { result } = renderHook(() =>
    //   useYNode({
    //     currentYWorkflow: yWorkflow,
    //     handleWorkflowsRemove: () => {},
    //     undoTrackerActionWrapper: () => {},
    //   }),
    // );
    // const newNodes: Node[] = [
    //   { id: "c", position: { x: 0, y: 0 }, data: { name: "Node C" } },
    // ];
    // act(() => {
    //   result.current.handleNodesUpdate(newNodes);
    // });
    // expect(yWorkflow.get("nodes")).toBeUndefined();
  });
});
