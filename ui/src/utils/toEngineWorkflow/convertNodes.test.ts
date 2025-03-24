import { describe, it, expect } from "vitest";

import type { EngineReadyNode, Node } from "@flow/types";

import { convertNodes } from "./convertNodes";

describe("convertNodes", () => {
  it("should return an empty array when input is undefined", () => {
    expect(convertNodes(undefined)).toEqual([]);
  });

  it("should filter out nodes with missing required properties", () => {
    const input = [
      {
        id: "1",
        type: "transformer",
        position: { x: 22, y: 22 },
        data: { officialName: "Node 1" },
      },
      { id: "2", type: "normal", position: { x: 22, y: 22 }, data: {} }, // Missing name
      { type: "normal", position: { x: 22, y: 22 }, data: { name: "Node 3" } }, // Missing id
      { id: "4", position: { x: 22, y: 22 }, data: { name: "Node 4" } }, // Missing type
    ];

    const expected: EngineReadyNode[] = [
      { id: "1", name: "Node 1", type: "action", action: "Node 1" },
    ];

    expect(convertNodes(input as Node[])).toEqual(expected);
  });

  it("should add subGraphId for subworkflow type nodes", () => {
    const input: Node[] = [
      {
        id: "1",
        type: "subworkflow",
        position: { x: 22, y: 22 },
        data: { officialName: "Subworkflow 1", subworkflowId: "01" },
      },
      {
        id: "2",
        type: "transformer",
        position: { x: 22, y: 22 },
        data: { officialName: "Normal Node" },
      },
    ];

    const expected: EngineReadyNode[] = [
      { id: "1", name: "Subworkflow 1", type: "subGraph", subGraphId: "01" },
      {
        id: "2",
        name: "Normal Node",
        type: "action",
        action: "Normal Node",
      },
    ];

    expect(convertNodes(input)).toEqual(expected);
  });
});
