import type { Edge, Node, Workflow } from "@flow/types";

import { buildReaderAttributeSuggestions } from "./readerAttributeSuggestions";

const readerNode = (id: string, fields: string[]): Node =>
  ({
    id,
    type: "reader",
    position: { x: 0, y: 0 },
    data: {
      officialName: "Reader",
      metadata: {
        schema: {
          ports: {
            default: {
              open: false,
              fields: fields.map((name) => ({
                name,
                type: "String",
              })),
            },
          },
        },
      },
    },
  }) as unknown as Node;

const plainNode = (id: string, type: string): Node =>
  ({
    id,
    type,
    position: { x: 0, y: 0 },
    data: { officialName: type },
  }) as unknown as Node;

const edge = (source: string, target: string): Edge =>
  ({
    id: `${source}-${target}`,
    source,
    target,
    sourceHandle: "default",
    targetHandle: "default",
  }) as unknown as Edge;

const subworkflowNode = (
  id: string,
  subworkflowId: string,
  pseudoInputs: { nodeId: string; portName: string }[],
  pseudoOutputs: { nodeId: string; portName: string }[],
): Node =>
  ({
    id,
    type: "subworkflow",
    position: { x: 0, y: 0 },
    data: {
      officialName: "Subworkflow",
      subworkflowId,
      pseudoInputs,
      pseudoOutputs,
    },
  }) as unknown as Node;

describe("buildReaderAttributeSuggestions", () => {
  // r1 → t1 → w1 ; r2 is an unconnected reader in the same graph.
  const workflows = [
    {
      id: "w1",
      nodes: [
        readerNode("r1", ["id", "name"]),
        plainNode("t1", "transformer"),
        plainNode("wr1", "writer"),
        readerNode("r2", ["geometry"]),
      ],
      edges: [edge("r1", "t1"), edge("t1", "wr1")],
    },
  ] as unknown as Workflow[];

  test("returns no suggestions without a target node", () => {
    expect(buildReaderAttributeSuggestions(workflows)).toEqual([]);
  });

  test("includes only readers upstream of the target node", () => {
    // t1 is downstream of r1, but not r2.
    expect(
      buildReaderAttributeSuggestions(workflows, "t1").map((s) => s.label),
    ).toEqual(["id", "name"]);
    // The writer further downstream still only sees r1.
    expect(
      buildReaderAttributeSuggestions(workflows, "wr1").map((s) => s.label),
    ).toEqual(["id", "name"]);
  });

  test("an unconnected reader sees only its own attributes", () => {
    expect(
      buildReaderAttributeSuggestions(workflows, "r2").map((s) => s.label),
    ).toEqual(["geometry"]);
  });

  test("dedupes attribute names across multiple upstream readers", () => {
    const merged = [
      {
        id: "w1",
        nodes: [
          readerNode("r1", ["id", "name"]),
          readerNode("r2", ["id", "geometry"]),
          plainNode("t1", "transformer"),
        ],
        edges: [edge("r1", "t1"), edge("r2", "t1")],
      },
    ] as unknown as Workflow[];

    expect(
      buildReaderAttributeSuggestions(merged, "t1").map((s) => s.label),
    ).toEqual(["id", "name", "geometry"]);
  });

  test("ignores readers without a probed schema", () => {
    const unprobed = [
      {
        id: "w1",
        nodes: [plainNode("r1", "reader"), plainNode("t1", "transformer")],
        edges: [edge("r1", "t1")],
      },
    ] as unknown as Workflow[];

    expect(buildReaderAttributeSuggestions(unprobed, "t1")).toEqual([]);
  });
});

describe("buildReaderAttributeSuggestions across subworkflow boundaries", () => {
  // Parent: reader R(pid) → subworkflow S → transformer P.
  // Child "child": ir → c ; r2(cid) → or.  S maps in=ir, out=or.
  const workflows = [
    {
      id: "main",
      nodes: [
        readerNode("R", ["pid"]),
        subworkflowNode(
          "S",
          "child",
          [{ nodeId: "ir", portName: "in" }],
          [{ nodeId: "or", portName: "out" }],
        ),
        plainNode("P", "transformer"),
      ],
      edges: [edge("R", "S"), edge("S", "P")],
    },
    {
      id: "child",
      nodes: [
        plainNode("ir", "transformer"),
        plainNode("c", "transformer"),
        readerNode("r2", ["cid"]),
        plainNode("or", "transformer"),
      ],
      edges: [edge("ir", "c"), edge("r2", "or")],
    },
  ] as unknown as Workflow[];

  test("a child node sees the parent reader feeding the subworkflow's input", () => {
    // c is downstream of the input router, fed from the parent's R — but NOT
    // the output-side reader r2 in the same subworkflow.
    expect(
      buildReaderAttributeSuggestions(workflows, "c").map((s) => s.label),
    ).toEqual(["pid"]);
  });

  test("a parent node downstream of the subworkflow sees the child's reader", () => {
    // P sees the child output reader r2 (via the output router) and the
    // flowed-through parent reader R.
    expect(
      buildReaderAttributeSuggestions(workflows, "P")
        .map((s) => s.label)
        .sort(),
    ).toEqual(["cid", "pid"]);
  });
});
