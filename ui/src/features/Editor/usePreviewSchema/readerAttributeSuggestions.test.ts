import type { Node, Workflow } from "@flow/types";

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
                presence: "always",
              })),
            },
          },
        },
      },
    },
  }) as unknown as Node;

describe("buildReaderAttributeSuggestions", () => {
  test("collects attribute names from all reader schemas", () => {
    const workflows = [
      { id: "w1", nodes: [readerNode("r1", ["id", "name"])] },
      { id: "w2", nodes: [readerNode("r2", ["geometry"])] },
    ] as unknown as Workflow[];

    const suggestions = buildReaderAttributeSuggestions(workflows);

    expect(suggestions.map((s) => s.label)).toEqual(["id", "name", "geometry"]);
    expect(suggestions[0]).toMatchObject({
      label: "id",
      insertText: "id",
      type: "variable",
    });
  });

  test("dedupes attribute names across readers", () => {
    const workflows = [
      {
        id: "w1",
        nodes: [readerNode("r1", ["id", "name"]), readerNode("r2", ["id"])],
      },
    ] as unknown as Workflow[];

    expect(
      buildReaderAttributeSuggestions(workflows).map((s) => s.label),
    ).toEqual(["id", "name"]);
  });

  test("ignores non-reader nodes and readers without a probed schema", () => {
    const transformer = {
      id: "t1",
      type: "transformer",
      position: { x: 0, y: 0 },
      data: { officialName: "T" },
    } as unknown as Node;
    const unprobedReader = {
      id: "r1",
      type: "reader",
      position: { x: 0, y: 0 },
      data: { officialName: "Reader" },
    } as unknown as Node;

    const workflows = [
      { id: "w1", nodes: [transformer, unprobedReader] },
    ] as unknown as Workflow[];

    expect(buildReaderAttributeSuggestions(workflows)).toEqual([]);
  });
});
