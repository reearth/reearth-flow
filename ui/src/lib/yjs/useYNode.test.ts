import { act, cleanup, renderHook } from "@testing-library/react";
import type { Dispatch, SetStateAction } from "react";
import * as Y from "yjs";

import { Node } from "@flow/types";

import { yWorkflowConstructor } from "./conversions";
import type { YNodesMap, YWorkflow } from "./types";
import useYNode from "./useYNode";

afterEach(() => {
  cleanup();
});

const createSelectedNodeIdsState = (initial: string[] = []) => {
  let selectedNodeIds = initial;
  const setSelectedNodeIds: Dispatch<SetStateAction<string[]>> = (action) => {
    selectedNodeIds =
      typeof action === "function"
        ? (action as (prev: string[]) => string[])(selectedNodeIds)
        : action;
  };
  return {
    setSelectedNodeIds,
    getSelectedNodeIds: () => selectedNodeIds,
  };
};

const buildNode = (id: string): Node => ({
  id,
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

  test("should update node position correctly", () => {
    const yDoc = new Y.Doc();
    const yWorkflows = yDoc.getMap<YWorkflow>("workflows");
    const yWorkflow = yWorkflowConstructor("workflow-1", "My Workflow", [
      buildNode("node-1"),
    ]);
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

    const { handleYNodesChange } = result.current;

    handleYNodesChange([
      { id: "node-1", type: "position", position: { x: 100, y: 200 } },
    ]);

    const yNodes = yWorkflow.get("nodes") as YNodesMap;
    const updatedNode = yNodes.get("node-1")?.toJSON() as Node;

    expect(updatedNode.position).toEqual({ x: 100, y: 200 });
  });

  test("should replace a node correctly", () => {
    const yDoc = new Y.Doc();
    const yWorkflows = yDoc.getMap<YWorkflow>("workflows");
    const yWorkflow = yWorkflowConstructor("workflow-1", "My Workflow", [
      buildNode("node-1"),
    ]);
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

    const { handleYNodesChange } = result.current;

    const replacementNode: Node = {
      ...buildNode("node-1"),
      data: {
        ...buildNode("node-1").data,
        officialName: "newOfficialName",
      },
    };

    handleYNodesChange([
      { id: "node-1", type: "replace", item: replacementNode },
    ]);

    const yNodes = yWorkflow.get("nodes") as YNodesMap;
    const updatedNode = yNodes.get("node-1")?.toJSON() as Node;

    expect(updatedNode.data.officialName).toBe("newOfficialName");
  });

  test("should update node dimensions correctly", () => {
    const yDoc = new Y.Doc();
    const yWorkflows = yDoc.getMap<YWorkflow>("workflows");
    const yWorkflow = yWorkflowConstructor("workflow-1", "My Workflow", [
      buildNode("node-1"),
    ]);
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

    const { handleYNodesChange } = result.current;

    handleYNodesChange([
      {
        id: "node-1",
        type: "dimensions",
        dimensions: { width: 300, height: 150 },
        setAttributes: true,
      },
    ]);

    const yNodes = yWorkflow.get("nodes") as YNodesMap;
    const updatedNode = yNodes.get("node-1")?.toJSON() as Node;

    expect(updatedNode.measured).toEqual({ width: 300, height: 150 });
    expect(updatedNode.style).toEqual({ width: "300px", height: "150px" });
  });

  test("should remove a node and clear its selection correctly", () => {
    const yDoc = new Y.Doc();
    const yWorkflows = yDoc.getMap<YWorkflow>("workflows");
    const yWorkflow = yWorkflowConstructor("workflow-1", "My Workflow", [
      buildNode("node-1"),
    ]);
    yWorkflows.set("workflow-1", yWorkflow);

    const { setSelectedNodeIds, getSelectedNodeIds } =
      createSelectedNodeIdsState(["node-1"]);

    const { result } = renderHook(() =>
      useYNode({
        currentYWorkflow: yWorkflow,
        yWorkflows,
        rawWorkflows: [],
        setSelectedNodeIds,
        undoTrackerActionWrapper: (callback) => act(callback),
        handleYWorkflowRemove: () => {},
      }),
    );

    const { handleYNodesChange } = result.current;

    handleYNodesChange([{ id: "node-1", type: "remove" }]);

    const yNodes = yWorkflow.get("nodes") as YNodesMap;

    expect(yNodes.has("node-1")).toBe(false);
    expect(getSelectedNodeIds()).toEqual([]);
  });

  test("should update selected node ids correctly", () => {
    const yDoc = new Y.Doc();
    const yWorkflows = yDoc.getMap<YWorkflow>("workflows");
    const yWorkflow = yWorkflowConstructor("workflow-1", "My Workflow", [
      buildNode("node-1"),
    ]);
    yWorkflows.set("workflow-1", yWorkflow);

    const { setSelectedNodeIds, getSelectedNodeIds } =
      createSelectedNodeIdsState([]);

    const { result } = renderHook(() =>
      useYNode({
        currentYWorkflow: yWorkflow,
        yWorkflows,
        rawWorkflows: [],
        setSelectedNodeIds,
        undoTrackerActionWrapper: (callback) => act(callback),
        handleYWorkflowRemove: () => {},
      }),
    );

    const { handleYNodesChange } = result.current;

    handleYNodesChange([{ id: "node-1", type: "select", selected: true }]);
    expect(getSelectedNodeIds()).toEqual(["node-1"]);

    handleYNodesChange([{ id: "node-1", type: "select", selected: false }]);
    expect(getSelectedNodeIds()).toEqual([]);
  });

  test("should select all nodes correctly", () => {
    const yDoc = new Y.Doc();
    const yWorkflows = yDoc.getMap<YWorkflow>("workflows");
    const yWorkflow = yWorkflowConstructor("workflow-1", "My Workflow", [
      buildNode("node-1"),
      buildNode("node-2"),
    ]);
    yWorkflows.set("workflow-1", yWorkflow);

    const { setSelectedNodeIds, getSelectedNodeIds } =
      createSelectedNodeIdsState([]);

    const { result } = renderHook(() =>
      useYNode({
        currentYWorkflow: yWorkflow,
        yWorkflows,
        rawWorkflows: [],
        setSelectedNodeIds,
        undoTrackerActionWrapper: (callback) => act(callback),
        handleYWorkflowRemove: () => {},
      }),
    );

    const { handleYNodesSelectAll } = result.current;

    handleYNodesSelectAll();

    expect(getSelectedNodeIds().sort()).toEqual(["node-1", "node-2"]);
  });

  test("should update node data correctly", () => {
    const yDoc = new Y.Doc();
    const yWorkflows = yDoc.getMap<YWorkflow>("workflows");
    const yWorkflow = yWorkflowConstructor("workflow-1", "My Workflow", [
      buildNode("node-1"),
    ]);
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

    const { handleYNodesDataUpdate } = result.current;

    handleYNodesDataUpdate([
      {
        nodeId: "node-1",
        updatedParams: { foo: "bar" },
        updatedCustomizations: { customName: "New Name" },
        isDisabled: true,
      },
    ]);

    const yNodes = yWorkflow.get("nodes") as YNodesMap;
    const updatedNode = yNodes.get("node-1")?.toJSON() as Node;

    expect(updatedNode.data.params).toEqual({ foo: "bar" });
    expect(updatedNode.data.customizations).toEqual({
      customName: "New Name",
    });
    expect(updatedNode.data.isDisabled).toBe(true);
  });

  test("should update node schema metadata correctly", () => {
    const yDoc = new Y.Doc();
    const yWorkflows = yDoc.getMap<YWorkflow>("workflows");
    const yWorkflow = yWorkflowConstructor("workflow-1", "My Workflow", [
      buildNode("node-1"),
    ]);
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

    const { handleYNodeSchemaUpdate } = result.current;

    handleYNodeSchemaUpdate("node-1", {
      jobId: "job-1",
      ports: { features: { open: true, fields: [] } },
    });

    const yNodes = yWorkflow.get("nodes") as YNodesMap;
    const yData = yNodes.get("node-1")?.get("data") as Y.Map<unknown>;

    expect(yData.get("nodeMetadata")).toEqual({
      schema: {
        ports: { features: { open: true, fields: [] } },
        jobId: "job-1",
      },
    });

    handleYNodeSchemaUpdate("node-1", undefined);

    expect(yData.get("nodeMetadata")).toEqual({ schema: undefined });
  });
});
