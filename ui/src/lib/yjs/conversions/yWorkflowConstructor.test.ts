import { cleanup } from "@testing-library/react";
import * as Y from "yjs";

import type { YEdgesArray, YNodesArray, YWorkflow } from "@flow/lib/yjs/types";
import type { Edge, Node } from "@flow/types";

import { reassembleEdge, reassembleNode } from "./rebuildWorkflow";
import { yWorkflowConstructor } from "./yWorkflowConstructor";

afterEach(() => {
  cleanup();
});

describe("yWorkflowConstructor", () => {
  test("should create a YWorkflow with the provided id and name", () => {
    const yDoc = new Y.Doc();
    const yWorkflows = yDoc.getArray<YWorkflow>("workflows");
    const id = "workflow-1";
    const name = "My Workflow";
    const yWorkflow = yWorkflowConstructor(id, name);

    yWorkflows.push([yWorkflow]);

    expect((yWorkflow.get("id") as Y.Text)?.toJSON()).toEqual(id);
    expect((yWorkflow.get("name") as Y.Text)?.toJSON()).toEqual(name);
  });

  test("should create a YWorkflow with the provided nodes and edges", () => {
    const yDoc = new Y.Doc();
    const yWorkflows = yDoc.getArray<YWorkflow>("workflows");
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
          content: "Content",
          backgroundColor: "#FFFFFF",
          textColor: "#000000",
        },
      },
    ];
    const edges: Edge[] = [
      { id: "edge-1", source: "node-1", target: "node-2" },
    ];
    const yWorkflow = yWorkflowConstructor(id, name, false, nodes, edges);
    yWorkflows.push([yWorkflow]);

    expect((yWorkflow.get("id") as Y.Text)?.toJSON()).toEqual(id);
    expect((yWorkflow.get("name") as Y.Text)?.toJSON()).toEqual(name);
    expect(
      (yWorkflow.get("nodes") as YNodesArray).map((yn) => reassembleNode(yn)),
    ).toEqual(nodes);
    expect(
      (yWorkflow.get("edges") as YEdgesArray).map((ye) => reassembleEdge(ye)),
    ).toEqual(edges);
  });

  it("should create a YWorkflow with empty nodes and edges if not provided", () => {
    const yDoc = new Y.Doc();
    const yWorkflows = yDoc.getArray<YWorkflow>("workflows");
    const id = "workflow-1";
    const name = "My Workflow";

    const yWorkflow = yWorkflowConstructor(id, name);
    yWorkflows.push([yWorkflow]);

    expect((yWorkflow.get("id") as Y.Text)?.toJSON()).toEqual(id);
    expect((yWorkflow.get("name") as Y.Text)?.toJSON()).toEqual(name);
    expect((yWorkflow.get("nodes") as YNodesArray)?.toArray()).toEqual([]);
    expect((yWorkflow.get("edges") as YEdgesArray)?.toArray()).toEqual([]);
  });
});
