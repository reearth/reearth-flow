import * as Y from "yjs";

import { Edge, Node } from "@flow/types";

import { YNode, YWorkflow } from "../types";

import { reassembleNode, rebuildWorkflow } from "./rebuildWorkflow";
import { yWorkflowConstructor } from "./yWorkflowConstructor";

describe("rebuildWorkflow", () => {
  test("should rebuild a workflow from a YWorkflow", () => {
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
          customizations: {
            customName: "Custom Name",
            content: "Content",
            backgroundColor: "#000000",
            titleColor: "#FFFFFF",
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

    yWorkflows.set(id, yWorkflow);

    const workflow = rebuildWorkflow(yWorkflow);

    expect(workflow.id).toEqual(id);
    expect(workflow.name).toEqual(name);
    expect(workflow.nodes).toEqual(nodes);
    expect(workflow.edges).toEqual(edges);
  });

  // Builds the exact poisoned-node shape decoded from the test-env doc:
  // a node integrated in a Y.Doc whose `position` map holds the given x/y.
  const makePoisonedYNode = (
    officialName: string,
    setPosition: (positionMap: Y.Map<unknown>) => void,
  ): YNode => {
    const yDoc = new Y.Doc();
    const yNodes = yDoc.getMap<Y.Map<unknown>>("nodes");
    const yNode = new Y.Map<unknown>();
    yNode.set("id", `node-${officialName}`);
    yNode.set("type", "transformer");
    yNode.set("dragging", false);
    const position = new Y.Map<unknown>();
    yNode.set("position", position);
    const data = new Y.Map<unknown>();
    data.set("officialName", officialName);
    yNode.set("data", data);
    yNodes.set(`node-${officialName}`, yNode);
    // Mutate the position AFTER integration, mirroring useYNode's raw
    // `existingPosition.set("x", change.position.x)` write path.
    setPosition(position);
    return yNode as unknown as YNode;
  };

  test("reassembleNode returns a finite position when the stored position is NaN", () => {
    // Reproduces the test-env room-open crash: screenToFlowPosition returns NaN
    // when a node is added before the canvas pane is measured; the raw useYNode
    // write persists it, and reassembleNode's `?? 0` does not catch NaN
    // (NaN ?? 0 === NaN), so a NaN reaches ReactFlow and triggers the render loop.
    const yNode = makePoisonedYNode("HorizontalReprojector", (position) => {
      position.set("x", NaN);
      position.set("y", NaN);
    });

    // Guard the reproduction: the stored value really is NaN.
    expect((yNode.get("position") as Y.Map<number>).get("x")).toBeNaN();

    const rebuilt = reassembleNode(yNode);

    expect(Number.isFinite(rebuilt.position.x)).toBe(true);
    expect(Number.isFinite(rebuilt.position.y)).toBe(true);
    expect(rebuilt.position).toEqual({ x: 0, y: 0 });
  });

  test("reassembleNode returns a finite position when the position map is empty", () => {
    // The sibling anomaly in the poisoned doc: a `position` Y.Map that exists
    // but has no x/y keys. reassembleNode must still yield a finite point.
    const yNode = makePoisonedYNode("VerticalReprojector", () => {
      // intentionally set no x/y keys
    });

    expect([...(yNode.get("position") as Y.Map<number>).keys()]).toEqual([]);

    const rebuilt = reassembleNode(yNode);

    expect(Number.isFinite(rebuilt.position.x)).toBe(true);
    expect(Number.isFinite(rebuilt.position.y)).toBe(true);
    expect(rebuilt.position).toEqual({ x: 0, y: 0 });
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
