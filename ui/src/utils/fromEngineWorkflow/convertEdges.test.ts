import { describe, it, expect } from "vitest";

import type { EngineReadyEdge } from "@flow/types";

import { convertEdges } from "./convertEdges";

describe("convertEdges", () => {
  it("should return an empty array when input is undefined", () => {
    const result = convertEdges(undefined);
    expect(result).toEqual([]);
  });

  it("should correctly convert a single edge", () => {
    const input: EngineReadyEdge[] = [
      {
        id: "edge1",
        from: "node1",
        to: "node2",
        fromPort: "output1",
        toPort: "input1",
      },
    ];

    const expected = [
      {
        id: "edge1",
        source: "node1",
        target: "node2",
        sourceHandle: "output1",
        targetHandle: "input1",
      },
    ];

    const result = convertEdges(input);
    expect(result).toEqual(expected);
  });

  it("should correctly convert multiple edges", () => {
    const input: EngineReadyEdge[] = [
      {
        id: "edge1",
        from: "node1",
        to: "node2",
        fromPort: "output1",
        toPort: "input1",
      },
      {
        id: "edge2",
        from: "node2",
        to: "node3",
        fromPort: "output2",
        toPort: "input2",
      },
    ];

    const expected = [
      {
        id: "edge1",
        source: "node1",
        target: "node2",
        sourceHandle: "output1",
        targetHandle: "input1",
      },
      {
        id: "edge2",
        source: "node2",
        target: "node3",
        sourceHandle: "output2",
        targetHandle: "input2",
      },
    ];

    const result = convertEdges(input);
    expect(result).toEqual(expected);
  });

  it("should preserve all edge properties during conversion", () => {
    const input: EngineReadyEdge[] = [
      {
        id: "edge1",
        from: "node1",
        to: "node2",
        fromPort: "output1",
        toPort: "input1",
      },
    ];

    const result = convertEdges(input);

    expect(result[0]).toHaveProperty("id", "edge1");
    expect(result[0]).toHaveProperty("source", "node1");
    expect(result[0]).toHaveProperty("target", "node2");
    expect(result[0]).toHaveProperty("sourceHandle", "output1");
    expect(result[0]).toHaveProperty("targetHandle", "input1");
  });

  it("should return an empty array when input is an empty array", () => {
    const result = convertEdges([]);
    expect(result).toEqual([]);
  });
});
