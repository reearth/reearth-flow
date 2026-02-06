import { cleanup } from "@testing-library/react";
import * as Y from "yjs";

import type { YEdgesMap, YNodesMap, YWorkflow } from "@flow/lib/yjs/types";
import type { Edge, Node } from "@flow/types";

import { reassembleEdge, reassembleNode } from "./rebuildWorkflow";
import { yWorkflowConstructor } from "./yWorkflowConstructor";

afterEach(() => {
  cleanup();
});

describe("yWorkflowConstructor", () => {
  test("should create a YWorkflow with the provided id and name", () => {
    const yDoc = new Y.Doc();
    const yWorkflows = yDoc.getMap<YWorkflow>("workflows");
    const id = "workflow-1";
    const name = "My Workflow";
    const yWorkflow = yWorkflowConstructor(id, name);

    yWorkflows.set(id, yWorkflow);

    expect(yWorkflow.get("id")?.toJSON()).toEqual(id);
    expect(yWorkflow.get("name")?.toJSON()).toEqual(name);
  });

  test("should create a YWorkflow with the provided nodes and edges", () => {
    const yDoc = new Y.Doc();
    const yWorkflows = yDoc.getMap<YWorkflow>("workflows");
    const id = "workflow-1";
    const name = "My Workflow";

    const nodes: Node[] = [
      {
        id: "node-1",
        type: "transformer",
        position: { x: 0, y: 0 },
        measured: { width: 100, height: 100 },
        dragging: false,
        data: {
          officialName: "Node 1",
          inputs: ["input1", "input2"],
          outputs: ["output1", "output2"],
          isCollapsed: false,
          isDisabled: false,
          params: {
            param1: "value1",
            param2: 2,
            param3: true,
            param4: null,
            param5: { key: "value" },
          },
          pseudoInputs: [
            { nodeId: "node-2", portName: "port1" },
            { nodeId: "node-3", portName: "port2" },
          ],
          pseudoOutputs: [
            { nodeId: "node-4", portName: "port3" },
            { nodeId: "node-5", portName: "port4" },
          ],
        },
      },
    ];
    const edges: Edge[] = [
      { id: "edge-1", source: "node-1", target: "node-2" },
    ];
    const yWorkflow = yWorkflowConstructor(id, name, nodes, edges);
    yWorkflows.set(id, yWorkflow);

    expect(yWorkflow.get("id")?.toJSON()).toEqual(id);
    expect(yWorkflow.get("name")?.toJSON()).toEqual(name);
    expect(
      Array.from(yWorkflow.get("nodes") as YNodesMap).map(([, yn]) =>
        reassembleNode(yn),
      ),
    ).toEqual(nodes);
    expect(
      Array.from(yWorkflow.get("edges") as YEdgesMap).map(([, ye]) =>
        reassembleEdge(ye),
      ),
    ).toEqual(edges);
  });

  it("should create a YWorkflow with empty nodes and edges if not provided", () => {
    const yDoc = new Y.Doc();
    const yWorkflows = yDoc.getArray<YWorkflow>("workflows");
    const id = "workflow-1";
    const name = "My Workflow";

    const yWorkflow = yWorkflowConstructor(id, name);
    yWorkflows.push([yWorkflow]);

    expect(yWorkflow.get("id")?.toJSON()).toEqual(id);
    expect(yWorkflow.get("name")?.toJSON()).toEqual(name);
    expect(Array.from(yWorkflow.get("nodes") as YNodesMap)).toEqual([]);
    expect(Array.from(yWorkflow.get("edges") as YEdgesMap)).toEqual([]);
  });

  describe("workflowPath parameter", () => {
    it("should set workflowPath on all nodes when provided", () => {
      const yDoc = new Y.Doc();
      const yWorkflows = yDoc.getMap<YWorkflow>("workflows");

      const nodes: Node[] = [
        {
          id: "node-1",
          type: "transformer",
          position: { x: 0, y: 0 },
          measured: { width: 100, height: 100 },
          dragging: false,
          data: {
            officialName: "Node 1",
            inputs: ["input1", "input2"],
            outputs: ["output1", "output2"],
            isCollapsed: false,
            isDisabled: false,
            params: {
              param1: "value1",
              param2: 2,
              param3: true,
              param4: null,
              param5: { key: "value" },
            },
            pseudoInputs: [
              { nodeId: "node-2", portName: "port1" },
              { nodeId: "node-3", portName: "port2" },
            ],
            pseudoOutputs: [
              { nodeId: "node-4", portName: "port3" },
              { nodeId: "node-5", portName: "port4" },
            ],
          },
        },
      ];

      const yWorkflow = yWorkflowConstructor(
        "wf-1",
        "Workflow",
        nodes,
        [],
        "sub-1",
      );
      yWorkflows.set("wf-1", yWorkflow);

      const yNodes = yWorkflow.get("nodes") as YNodesMap;
      const rebuiltNodes = Array.from(yNodes).map(([, yn]) =>
        reassembleNode(yn),
      );

      expect(rebuiltNodes[0].data.workflowPath).toBe("sub-1");
    });

    it("should set nested workflowPath with dot notation on all nodes", () => {
      const yDoc = new Y.Doc();
      const yWorkflows = yDoc.getMap<YWorkflow>("workflows");

      const nodes: Node[] = [
        {
          id: "node-1",
          type: "transformer",
          position: { x: 0, y: 0 },
          measured: { width: 100, height: 100 },
          dragging: false,
          data: {
            officialName: "Node 1",
            inputs: ["input1", "input2"],
            outputs: ["output1", "output2"],
            isCollapsed: false,
            isDisabled: false,
            params: {
              param1: "value1",
              param2: 2,
              param3: true,
              param4: null,
              param5: { key: "value" },
            },
            pseudoInputs: [
              { nodeId: "node-2", portName: "port1" },
              { nodeId: "node-3", portName: "port2" },
            ],
            pseudoOutputs: [
              { nodeId: "node-4", portName: "port3" },
              { nodeId: "node-5", portName: "port4" },
            ],
          },
        },
      ];

      const yWorkflow = yWorkflowConstructor(
        "wf-deep",
        "Deep Workflow",
        nodes,
        [],
        "sub-1.sub-2",
      );
      yWorkflows.set("wf-deep", yWorkflow);

      const yNodes = yWorkflow.get("nodes") as YNodesMap;
      const rebuiltNode = reassembleNode(
        Array.from(yNodes).map(([, yn]) => yn)[0],
      );

      expect(rebuiltNode.data.workflowPath).toBe("sub-1.sub-2");
    });

    it("should not set workflowPath when not provided", () => {
      const yDoc = new Y.Doc();
      const yWorkflows = yDoc.getMap<YWorkflow>("workflows");

      const nodes: Node[] = [
        {
          id: "node-1",
          type: "transformer",
          position: { x: 0, y: 0 },
          measured: { width: 100, height: 100 },
          dragging: false,
          data: {
            officialName: "Node 1",
            inputs: ["input1", "input2"],
            outputs: ["output1", "output2"],
            isCollapsed: false,
            isDisabled: false,
            params: {
              param1: "value1",
              param2: 2,
              param3: true,
              param4: null,
              param5: { key: "value" },
            },
            pseudoInputs: [
              { nodeId: "node-2", portName: "port1" },
              { nodeId: "node-3", portName: "port2" },
            ],
            pseudoOutputs: [
              { nodeId: "node-4", portName: "port3" },
              { nodeId: "node-5", portName: "port4" },
            ],
          },
        },
      ];

      const yWorkflow = yWorkflowConstructor("wf-1", "Workflow", nodes, []);
      yWorkflows.set("wf-1", yWorkflow);

      const yNodes = yWorkflow.get("nodes") as YNodesMap;
      const rebuiltNode = reassembleNode(
        Array.from(yNodes).map(([, yn]) => yn)[0],
      );

      expect(rebuiltNode.data.workflowPath).toBeUndefined();
    });

    it("should override existing workflowPath on nodes", () => {
      const yDoc = new Y.Doc();
      const yWorkflows = yDoc.getMap<YWorkflow>("workflows");

      const nodes: Node[] = [
        {
          id: "node-1",
          type: "transformer",
          position: { x: 0, y: 0 },
          measured: { width: 100, height: 100 },
          dragging: false,
          data: {
            officialName: "Node 1",
            inputs: ["input1", "input2"],
            outputs: ["output1", "output2"],
            isCollapsed: false,
            isDisabled: false,
            workflowPath: "old-path",
            params: {
              param1: "value1",
              param2: 2,
              param3: true,
              param4: null,
              param5: { key: "value" },
            },
            pseudoInputs: [
              { nodeId: "node-2", portName: "port1" },
              { nodeId: "node-3", portName: "port2" },
            ],
            pseudoOutputs: [
              { nodeId: "node-4", portName: "port3" },
              { nodeId: "node-5", portName: "port4" },
            ],
          },
        },
      ];

      const yWorkflow = yWorkflowConstructor(
        "wf-1",
        "Workflow",
        nodes,
        [],
        "new-path",
      );
      yWorkflows.set("wf-1", yWorkflow);

      const yNodes = yWorkflow.get("nodes") as YNodesMap;
      const rebuiltNode = reassembleNode(
        Array.from(yNodes).map(([, yn]) => yn)[0],
      );

      expect(rebuiltNode.data.workflowPath).toBe("new-path");
    });

    it("should set empty string workflowPath for entry graph nodes", () => {
      const yDoc = new Y.Doc();
      const yWorkflows = yDoc.getMap<YWorkflow>("workflows");

      const nodes: Node[] = [
        {
          id: "node-1",
          type: "transformer",
          position: { x: 0, y: 0 },
          measured: { width: 100, height: 100 },
          dragging: false,
          data: {
            officialName: "Node 1",
            inputs: ["input1", "input2"],
            outputs: ["output1", "output2"],
            isCollapsed: false,
            isDisabled: false,
            workflowPath: "old-path",
            params: {
              param1: "value1",
              param2: 2,
              param3: true,
              param4: null,
              param5: { key: "value" },
            },
            pseudoInputs: [
              { nodeId: "node-2", portName: "port1" },
              { nodeId: "node-3", portName: "port2" },
            ],
            pseudoOutputs: [
              { nodeId: "node-4", portName: "port3" },
              { nodeId: "node-5", portName: "port4" },
            ],
          },
        },
      ];

      const yWorkflow = yWorkflowConstructor("entry", "Entry", nodes, [], "");
      yWorkflows.set("entry", yWorkflow);

      const yNodes = yWorkflow.get("nodes") as YNodesMap;
      const rebuiltNode = reassembleNode(
        Array.from(yNodes).map(([, yn]) => yn)[0],
      );

      // Empty string from computeWorkflowPath for entry graph results in
      // toYjsText returning undefined, so workflowPath should be undefined
      expect(rebuiltNode.data.workflowPath).toBeUndefined();
    });
  });
});
