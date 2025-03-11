import * as Y from "yjs";

import { Edge, Node } from "@flow/types";

import { YWorkflow } from "../types";

import { rebuildWorkflow } from "./rebuildWorkflow";
import { yWorkflowConstructor } from "./yWorkflowConstructor";

describe("rebuildWorkflow", () => {
  test("should rebuild a workflow from a YWorkflow", () => {
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
          customizations: {
            customName: "Custom Name",
            content: "Content",
            backgroundColor: "#000000",
            textColor: "#FFFFFF",
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
      {
        id: "edge-1",
        source: "node-1",
        target: "node-2",
        sourceHandle: "output1",
        targetHandle: "input1",
      },
    ];

    const yWorkflow = yWorkflowConstructor(id, name, nodes, edges);

    yWorkflows.push([yWorkflow]);

    const workflow = rebuildWorkflow(yWorkflow);

    expect(workflow.id).toEqual(id);
    expect(workflow.name).toEqual(name);
    expect(workflow.nodes).toEqual(nodes);
    expect(workflow.edges).toEqual(edges);
  });

  test("should handle empty workflow", () => {
    const yDoc = new Y.Doc();
    const yWorkflows = yDoc.getArray<YWorkflow>("workflows");
    const id = "empty-workflow";
    const name = "Empty Workflow";

    const yWorkflow = yWorkflowConstructor(id, name, [], []);
    yWorkflows.push([yWorkflow]);

    const workflow = rebuildWorkflow(yWorkflow);

    expect(workflow.id).toEqual(id);
    expect(workflow.name).toEqual(name);
    expect(workflow.nodes).toEqual([]);
    expect(workflow.edges).toEqual([]);
  });
});
