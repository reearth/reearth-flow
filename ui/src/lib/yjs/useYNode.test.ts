import { act, cleanup, renderHook } from "@testing-library/react";
import * as Y from "yjs";

import { Node } from "@flow/types";

import { yWorkflowConstructor } from "./conversions";
import type { YNodesMap, YWorkflow } from "./types";
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
        params: node.data.params,
        customizations: node.data.customizations,
        isCollapsed: node.data.isCollapsed ?? false,
        isDisabled: node.data.isDisabled ?? false,
      },
    }));

    expect(n).toEqual(expectedNodes);
  });
});
