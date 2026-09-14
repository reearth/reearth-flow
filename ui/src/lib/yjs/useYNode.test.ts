import { act, cleanup, renderHook } from "@testing-library/react";
import * as Y from "yjs";

import { Node } from "@flow/types";

import { yWorkflowConstructor } from "./conversions";
import type { YEdgesMap, YNodesMap, YWorkflow } from "./types";
import useYNode from "./useYNode";

afterEach(() => {
  cleanup();
});

describe("useYNode", () => {
  test("should add nodes correctly", () => {
    const yDoc = new Y.Doc();
    const yWorkflows = yDoc.getMap<YWorkflow>("workflows");
    const yWorkflow = yWorkflowConstructor("workflow-1", "My Workflow");

    yWorkflows.set("workflow-1", yWorkflow);

    const { result } = renderHook(() =>
      useYNode({
        currentYWorkflow: yWorkflow,
        yWorkflows,
        rawWorkflows: [],
        setSelectedNodeIds: () => {},
        undoTrackerActionWrapper: (callback) => act(callback),
        handleYWorkflowRemove: () => {},
      }),
    );

    const { handleYNodesAdd } = result.current;

    const newNodes: Node[] = [
      {
        id: "node-1",
        type: "batch",
        position: { x: 0, y: 0 },
        measured: { width: 0, height: 0 },
        data: {
          officialName: "officialName",
          inputs: ["input1"],
          outputs: ["output1"],
          params: {},
          customizations: {},
          isCollapsed: true,
          isDisabled: false,
          pseudoInputs: [],
          pseudoOutputs: [],
        },
        style: { width: 0, height: 0 },
      },
    ];

    handleYNodesAdd(newNodes);

    const yNodes = yWorkflow.get("nodes") as YNodesMap;

    const n = Object.values(yNodes.toJSON()) as Node[];

    const expectedNodes = newNodes.map((node) => ({
      ...node,
      dragging: false,
      data: {
        // NOTE: we expect the empty fields to be omitted (pseudoInputs, pseudoOutputs)
        officialName: node.data.officialName,
        inputs: node.data.inputs,
        outputs: node.data.outputs,
        workflowPath: node.data.workflowPath,
        params: node.data.params,
        customizations: node.data.customizations,
        isCollapsed: node.data.isCollapsed ?? false,
        isDisabled: node.data.isDisabled ?? false,
      },
    }));

    expect(n).toEqual(expectedNodes);
  });
});

describe("useYNode edge cleanup", () => {
  test("removing a node removes the edges attached to it", () => {
    // Callers emit their own edge removals, but every one of them works from a
    // snapshot of some kind. This is the backstop that makes edge cleanup a
    // property of deleting the node rather than of remembering to ask.
    const yDoc = new Y.Doc();
    const yWorkflows = yDoc.getMap<YWorkflow>("workflows");
    const node = (id: string): Node => ({
      id,
      type: "transformer",
      position: { x: 0, y: 0 },
      data: { officialName: id, inputs: ["features"], outputs: ["features"] },
    });
    const yWorkflow = yWorkflowConstructor(
      "workflow-1",
      "My Workflow",
      [node("a"), node("b"), node("c")],
      [
        { id: "a-b", source: "a", target: "b" },
        { id: "b-c", source: "b", target: "c" },
        { id: "a-c", source: "a", target: "c" },
      ],
    );
    yWorkflows.set("workflow-1", yWorkflow);

    const { result } = renderHook(() =>
      useYNode({
        currentYWorkflow: yWorkflow,
        yWorkflows,
        rawWorkflows: [],
        setSelectedNodeIds: () => {},
        undoTrackerActionWrapper: (callback) => act(callback),
      }),
    );

    // No accompanying edge changes - the node change arrives on its own.
    result.current.handleYNodesChange([{ id: "b", type: "remove" }]);

    const remainingEdges = Object.keys(
      (yWorkflow.get("edges") as YEdgesMap).toJSON(),
    );
    expect(remainingEdges).toEqual(["a-c"]);
    expect(
      Object.keys((yWorkflow.get("nodes") as YNodesMap).toJSON()).sort(),
    ).toEqual(["a", "c"]);
  });
});
