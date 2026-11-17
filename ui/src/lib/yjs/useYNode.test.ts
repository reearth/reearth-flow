import { act, cleanup, renderHook } from "@testing-library/react";
import * as Y from "yjs";

import type { Node, NodeChange } from "@flow/types";

import { yWorkflowConstructor } from "./conversions";
import type { YNode, YNodesMap, YWorkflow } from "./types";
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

  describe("handleYNodesChange geometry writes", () => {
    const setup = ({ withStyle = true } = {}) => {
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
          undoTrackerActionWrapper: (callback) => yDoc.transact(callback),
          handleYWorkflowRemove: () => {},
        }),
      );

      result.current.handleYNodesAdd([
        {
          id: "node-1",
          type: "batch",
          position: { x: 10, y: 20 },
          measured: { width: 300, height: 200 },
          data: { officialName: "batch" },
          ...(withStyle ? { style: { width: "300px", height: "200px" } } : {}),
        },
      ]);

      const yNode = (yWorkflow.get("nodes") as YNodesMap).get("node-1");
      if (!yNode) throw new Error("node was not added");

      let updates = 0;
      yDoc.on("update", () => updates++);

      return {
        yNode,
        applyChanges: (changes: NodeChange[]) =>
          result.current.handleYNodesChange(changes),
        getUpdateCount: () => updates,
      };
    };

    const geometry = (yNode: YNode, key: string) => {
      const value = yNode.get(key);
      return value instanceof Y.Map ? value.toJSON() : undefined;
    };

    test("unchanged dimensions do not update the document", () => {
      const { applyChanges, getUpdateCount } = setup();

      applyChanges([
        {
          id: "node-1",
          type: "dimensions",
          dimensions: { width: 300, height: 200 },
        },
        {
          id: "node-1",
          type: "dimensions",
          dimensions: { width: 300, height: 200 },
          setAttributes: true,
        },
      ]);

      expect(getUpdateCount()).toBe(0);
    });

    test("changed dimensions are written", () => {
      const { yNode, applyChanges, getUpdateCount } = setup();

      applyChanges([
        {
          id: "node-1",
          type: "dimensions",
          dimensions: { width: 300, height: 250 },
          setAttributes: true,
        },
      ]);

      expect(getUpdateCount()).toBe(1);
      expect(geometry(yNode, "measured")).toEqual({ width: 300, height: 250 });
      expect(geometry(yNode, "style")).toEqual({
        width: "300px",
        height: "250px",
      });
    });

    test("only the differing dimension is rewritten", () => {
      const { yNode, applyChanges } = setup();
      const measured = yNode.get("measured");
      if (!(measured instanceof Y.Map)) throw new Error("no measured map");
      const keysChanged: string[][] = [];
      measured.observe((event) => keysChanged.push([...event.keysChanged]));

      applyChanges([
        {
          id: "node-1",
          type: "dimensions",
          dimensions: { width: 320, height: 200 },
        },
      ]);

      expect(keysChanged).toEqual([["width"]]);
    });

    test("a missing style map is created when attributes are set", () => {
      const { yNode, applyChanges, getUpdateCount } = setup({
        withStyle: false,
      });
      expect(yNode.get("style")).toBeUndefined();

      applyChanges([
        {
          id: "node-1",
          type: "dimensions",
          dimensions: { width: 300, height: 200 },
          setAttributes: true,
        },
      ]);

      expect(getUpdateCount()).toBe(1);
      expect(geometry(yNode, "style")).toEqual({
        width: "300px",
        height: "200px",
      });
    });

    test("an unchanged position does not update the document", () => {
      const { applyChanges, getUpdateCount } = setup();

      // React Flow repeats the final position when a drag ends.
      applyChanges([
        {
          id: "node-1",
          type: "position",
          position: { x: 10, y: 20 },
          dragging: false,
        },
      ]);

      expect(getUpdateCount()).toBe(0);
    });

    test("a changed position is written", () => {
      const { yNode, applyChanges, getUpdateCount } = setup();

      applyChanges([
        {
          id: "node-1",
          type: "position",
          position: { x: 15, y: 20 },
          dragging: true,
        },
      ]);

      expect(getUpdateCount()).toBe(1);
      expect(geometry(yNode, "position")).toEqual({ x: 15, y: 20 });
    });
  });
});
