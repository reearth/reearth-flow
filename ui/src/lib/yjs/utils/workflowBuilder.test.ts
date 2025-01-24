import { cleanup } from "@testing-library/react";
import * as Y from "yjs";

import { Edge, Node } from "@flow/types";

import { reassembleEdge, reassembleNode } from "./convertToRawWorkflow";
import {
  yWorkflowBuilder,
  YNodesArray,
  YEdgesArray,
  YWorkflow,
} from "./workflowBuilder";

afterEach(() => {
  cleanup();
});

describe("yWorkflowBuilder", () => {
  test("should create a YWorkflow with the provided id and name", () => {
    const yDoc = new Y.Doc();
    const yWorkflows = yDoc.getArray<YWorkflow>("workflows");
    const id = "workflow-1";
    const name = "My Workflow";
    const yWorkflow = yWorkflowBuilder(id, name);

    yWorkflows.push([yWorkflow]);

    expect(yWorkflow.get("id")?.toJSON()).toEqual(id);
    expect(yWorkflow.get("name")?.toJSON()).toEqual(name);
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
        data: { officialName: "Node 1" },
      },
    ];
    const edges: Edge[] = [
      { id: "edge-1", source: "node-1", target: "node-2" },
    ];
    const yWorkflow = yWorkflowBuilder(id, name, nodes, edges);
    yWorkflows.push([yWorkflow]);

    expect(yWorkflow.get("id")?.toJSON()).toEqual(id);
    expect(yWorkflow.get("name")?.toJSON()).toEqual(name);
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

    const yWorkflow = yWorkflowBuilder(id, name);
    yWorkflows.push([yWorkflow]);

    expect(yWorkflow.get("id")?.toJSON()).toEqual(id);
    expect(yWorkflow.get("name")?.toJSON()).toEqual(name);
    expect((yWorkflow.get("nodes") as YNodesArray)?.toArray()).toEqual([]);
    expect((yWorkflow.get("edges") as YEdgesArray)?.toArray()).toEqual([]);
  });
});
