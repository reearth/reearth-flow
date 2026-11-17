import { describe, expect, it } from "vitest";

import type { Edge, Node, NodeData, NodeType, Workflow } from "@flow/types";

import { createEngineReadyWorkflow } from "./engineReadyWorkflow";
import {
  buildRunGraph,
  downstreamOf,
  lastRunCovers,
  runnableNodeIds,
  trimWorkflowsToNode,
} from "./trimToNode";

const node = (
  id: string,
  type: NodeType = "transformer",
  data: Partial<NodeData> = {},
): Node => ({
  id,
  type,
  position: { x: 0, y: 0 },
  data: { officialName: id, ...data },
});

const subworkflow = (
  id: string,
  inputs: [string, string][],
  outputs: [string, string][],
): Node =>
  node(id, "subworkflow", {
    subworkflowId: id,
    pseudoInputs: inputs.map(([nodeId, portName]) => ({ nodeId, portName })),
    pseudoOutputs: outputs.map(([nodeId, portName]) => ({ nodeId, portName })),
  });

const edge = (
  source: string,
  target: string,
  sourceHandle?: string,
  targetHandle?: string,
): Edge => ({
  id: `${source}->${target}${targetHandle ? `:${targetHandle}` : ""}`,
  source,
  target,
  sourceHandle,
  targetHandle,
});

const main = (nodes: Node[], edges: Edge[]): Workflow => ({
  id: "main",
  name: "Main",
  nodes,
  edges,
});

const graph = (id: string, nodes: Node[], edges: Edge[]): Workflow => ({
  id,
  name: id,
  nodes,
  edges,
});

const keptIds = (workflows: Workflow[], target: string) => {
  const result = trimWorkflowsToNode(workflows, target);
  return [...(result?.keptNodeIds ?? [])].sort();
};

const workflowOf = (workflows: Workflow[] | undefined, id: string) =>
  workflows?.find((w) => w.id === id);

const nodeIdsIn = (workflow?: Workflow) =>
  workflow?.nodes?.map((n) => n.id).sort();

const edgeIdsIn = (workflow?: Workflow) =>
  workflow?.edges?.map((e) => e.id).sort();

describe("trimWorkflowsToNode", () => {
  it("keeps what feeds the target in a chain and drops what follows", () => {
    const workflows = [
      main(
        [node("R", "reader"), node("A"), node("B"), node("W", "writer")],
        [edge("R", "A"), edge("A", "B"), edge("B", "W")],
      ),
    ];

    const result = trimWorkflowsToNode(workflows, "A");

    expect(nodeIdsIn(result?.workflows[0])).toEqual(["A", "R"]);
    expect(edgeIdsIn(result?.workflows[0])).toEqual(["R->A"]);
  });

  it("drops a side branch that ends in a writer", () => {
    const workflows = [
      main(
        [node("R", "reader"), node("A"), node("W", "writer"), node("B")],
        [edge("R", "A"), edge("A", "W"), edge("R", "B")],
      ),
    ];

    expect(keptIds(workflows, "B")).toEqual(["B", "R"]);
  });

  it("keeps every input of a node with several", () => {
    const workflows = [
      main(
        [
          node("R1", "reader"),
          node("R2", "reader"),
          node("J"),
          node("W", "writer"),
        ],
        [
          edge("R1", "J", undefined, "left"),
          edge("R2", "J", undefined, "right"),
          edge("J", "W"),
        ],
      ),
    ];

    const result = trimWorkflowsToNode(workflows, "J");

    expect(nodeIdsIn(result?.workflows[0])).toEqual(["J", "R1", "R2"]);
    expect(edgeIdsIn(result?.workflows[0])).toEqual([
      "R1->J:left",
      "R2->J:right",
    ]);
  });

  it("keeps a writer as the target", () => {
    const workflows = [
      main(
        [
          node("R", "reader"),
          node("A"),
          node("W1", "writer"),
          node("W2", "writer"),
        ],
        [edge("R", "A"), edge("A", "W1"), edge("A", "W2")],
      ),
    ];

    expect(keptIds(workflows, "W1")).toEqual(["A", "R", "W1"]);
  });

  it("follows a target inside a subworkflow out to the parent", () => {
    const workflows = [
      main(
        [
          node("R", "reader"),
          subworkflow("S", [["IR", "in"]], [["OR", "out"]]),
          node("W", "writer"),
          node("X"),
          node("W2", "writer"),
        ],
        [
          edge("R", "S", undefined, "in"),
          edge("S", "W", "out"),
          edge("R", "X"),
          edge("X", "W2"),
        ],
      ),
      graph(
        "S",
        [node("IR"), node("F"), node("T"), node("OR")],
        [edge("IR", "F"), edge("F", "T"), edge("T", "OR")],
      ),
    ];

    const result = trimWorkflowsToNode(workflows, "F");

    expect(nodeIdsIn(workflowOf(result?.workflows, "main"))).toEqual([
      "R",
      "S",
    ]);
    expect(edgeIdsIn(workflowOf(result?.workflows, "main"))).toEqual([
      "R->S:in",
    ]);
    expect(nodeIdsIn(workflowOf(result?.workflows, "S"))).toEqual(["F", "IR"]);
    expect(edgeIdsIn(workflowOf(result?.workflows, "S"))).toEqual(["IR->F"]);
  });

  it("enters an upstream subworkflow only through the port that is used", () => {
    const workflows = [
      main(
        [
          node("R", "reader"),
          subworkflow("S", [["IR", "in"]], [["OR", "out"]]),
          node("A"),
        ],
        [edge("R", "S", undefined, "in"), edge("S", "A", "out")],
      ),
      graph(
        "S",
        [node("IR"), node("T"), node("OR"), node("WS", "writer")],
        [edge("IR", "T"), edge("T", "OR"), edge("IR", "WS")],
      ),
    ];

    const result = trimWorkflowsToNode(workflows, "A");

    expect(nodeIdsIn(workflowOf(result?.workflows, "S"))).toEqual([
      "IR",
      "OR",
      "T",
    ]);
    expect(nodeIdsIn(workflowOf(result?.workflows, "main"))).toEqual([
      "A",
      "R",
      "S",
    ]);
  });

  it("climbs out of nested subworkflows", () => {
    const workflows = [
      main(
        [
          node("R", "reader"),
          subworkflow("S1", [["IR1", "in"]], []),
          node("W", "writer"),
        ],
        [edge("R", "S1", undefined, "in"), edge("R", "W")],
      ),
      graph(
        "S1",
        [node("IR1"), subworkflow("S2", [["IR2", "in2"]], [])],
        [edge("IR1", "S2", undefined, "in2")],
      ),
      graph("S2", [node("IR2"), node("F")], [edge("IR2", "F")]),
    ];

    const result = trimWorkflowsToNode(workflows, "F");

    expect(result?.workflows.map((w) => w.id).sort()).toEqual([
      "S1",
      "S2",
      "main",
    ]);
    expect([...(result?.keptNodeIds ?? [])].sort()).toEqual([
      "F",
      "IR1",
      "IR2",
      "R",
      "S1",
      "S2",
    ]);
  });

  it("keeps the containers of a target fed only from inside its subworkflow", () => {
    const workflows = [
      main(
        [
          node("R", "reader"),
          subworkflow("S", [["IR", "in"]], []),
          node("W", "writer"),
        ],
        [edge("R", "S", undefined, "in"), edge("R", "W")],
      ),
      graph(
        "S",
        [node("IR"), node("R2", "reader"), node("F")],
        [edge("R2", "F")],
      ),
    ];

    const result = trimWorkflowsToNode(workflows, "F");

    expect(nodeIdsIn(workflowOf(result?.workflows, "main"))).toEqual(["S"]);
    expect(edgeIdsIn(workflowOf(result?.workflows, "main"))).toEqual([]);
    expect(nodeIdsIn(workflowOf(result?.workflows, "S"))).toEqual(["F", "R2"]);
  });

  it("drops a subworkflow input port the target does not depend on", () => {
    const workflows = [
      main(
        [
          node("R1", "reader"),
          node("R2", "reader"),
          subworkflow(
            "S",
            [
              ["IR1", "p1"],
              ["IR2", "p2"],
            ],
            [],
          ),
        ],
        [edge("R1", "S", undefined, "p1"), edge("R2", "S", undefined, "p2")],
      ),
      graph(
        "S",
        [node("IR1"), node("IR2"), node("F"), node("G")],
        [edge("IR1", "F"), edge("IR2", "G")],
      ),
    ];

    const result = trimWorkflowsToNode(workflows, "F");

    expect(nodeIdsIn(workflowOf(result?.workflows, "main"))).toEqual([
      "R1",
      "S",
    ]);
    expect(edgeIdsIn(workflowOf(result?.workflows, "main"))).toEqual([
      "R1->S:p1",
    ]);
    expect(nodeIdsIn(workflowOf(result?.workflows, "S"))).toEqual(["F", "IR1"]);
  });

  it("keeps everything that feeds a subworkflow node as the target", () => {
    const workflows = [
      main(
        [
          node("R", "reader"),
          subworkflow("S", [["IR", "in"]], [["OR", "out"]]),
          node("W", "writer"),
        ],
        [edge("R", "S", undefined, "in"), edge("S", "W", "out")],
      ),
      graph(
        "S",
        [node("IR"), node("T"), node("OR"), node("WS", "writer")],
        [edge("IR", "T"), edge("T", "OR"), edge("IR", "WS")],
      ),
    ];

    expect(keptIds(workflows, "S")).toEqual(["IR", "OR", "R", "S", "T"]);
  });

  it("treats a disabled action as a break in the chain", () => {
    const workflows = [
      main(
        [
          node("R", "reader"),
          node("D", "transformer", { isDisabled: true }),
          node("R2", "reader"),
          node("A"),
        ],
        [edge("R", "D"), edge("D", "A"), edge("R2", "A")],
      ),
    ];

    expect(keptIds(workflows, "A")).toEqual(["A", "R2"]);
  });

  it("returns nothing for a target that cannot run", () => {
    const workflows = [
      main(
        [node("N", "note"), node("D", "transformer", { isDisabled: true })],
        [],
      ),
    ];

    expect(trimWorkflowsToNode(workflows, "N")).toBeUndefined();
    expect(trimWorkflowsToNode(workflows, "D")).toBeUndefined();
    expect(trimWorkflowsToNode(workflows, "missing")).toBeUndefined();
  });

  it("produces a workflow the engine conversion accepts", () => {
    const workflows = [
      main(
        [
          node("R", "reader"),
          subworkflow("S", [["IR", "in"]], [["OR", "out"]]),
          node("W", "writer"),
        ],
        [edge("R", "S", undefined, "in"), edge("S", "W", "out")],
      ),
      graph(
        "S",
        [node("IR"), node("F"), node("OR")],
        [edge("IR", "F"), edge("F", "OR")],
      ),
    ];

    const trimmed = trimWorkflowsToNode(workflows, "F");
    const engineWorkflow = createEngineReadyWorkflow(
      "project",
      [],
      trimmed?.workflows,
    );

    const entry = engineWorkflow?.graphs.find(
      (g) => g.id === engineWorkflow.entryGraphId,
    );
    const sub = engineWorkflow?.graphs.find((g) => g.id === "S");
    expect(entry?.nodes.map((n) => n.id).sort()).toEqual(["R", "S"]);
    expect(entry?.edges).toEqual([
      { id: "R->S:in", from: "R", to: "S", fromPort: "features", toPort: "in" },
    ]);
    expect(sub?.nodes.map((n) => n.id).sort()).toEqual(["F", "IR"]);
  });
});

describe("downstreamOf", () => {
  it("crosses into and out of subworkflows by port", () => {
    const workflows = [
      main(
        [
          node("R", "reader"),
          subworkflow("S", [["IR", "in"]], [["OR", "out"]]),
          node("A"),
        ],
        [edge("R", "S", undefined, "in"), edge("S", "A", "out")],
      ),
      graph(
        "S",
        [node("IR"), node("T"), node("OR")],
        [edge("IR", "T"), edge("T", "OR")],
      ),
    ];

    expect([...downstreamOf(buildRunGraph(workflows), "T")].sort()).toEqual([
      "A",
      "OR",
      "T",
    ]);
  });
});

describe("lastRunCovers", () => {
  const workflows = [
    main(
      [node("R1", "reader"), node("A"), node("R2", "reader"), node("J")],
      [
        edge("R1", "A"),
        edge("A", "J", undefined, "left"),
        edge("R2", "J", undefined, "right"),
      ],
    ),
  ];
  const runGraph = buildRunGraph(workflows);

  it("refuses a start whose skipped inputs the last run never produced", () => {
    expect(lastRunCovers(runGraph, ["R1", "A"], "J")).toBe(false);
  });

  it("refuses a start when a later action needs an input the last run skipped", () => {
    expect(lastRunCovers(runGraph, ["R1", "A"], "A")).toBe(false);
  });

  it("accepts a start whose skipped inputs the last run produced", () => {
    expect(lastRunCovers(runGraph, ["R1", "A", "R2", "J"], "A")).toBe(true);
    expect(lastRunCovers(runGraph, ["R1", "A", "R2", "J"], "J")).toBe(true);
  });
});

describe("runnableNodeIds", () => {
  it("lists actions that will run, not notes or disabled ones", () => {
    const workflows = [
      main(
        [
          node("R", "reader"),
          node("N", "note"),
          node("D", "transformer", { isDisabled: true }),
        ],
        [],
      ),
    ];

    expect(runnableNodeIds(workflows)).toEqual(["R"]);
  });
});
