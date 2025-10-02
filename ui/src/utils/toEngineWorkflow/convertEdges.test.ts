import { describe, it, expect } from "vitest";

import type { Edge } from "@flow/types";

import { convertEdges } from "./convertEdges";

describe("convertEdges", () => {
  it("should return an empty array when input is undefined", () => {
    const enabledNodeIds = new Set(["A", "B"]);
    expect(convertEdges(enabledNodeIds, undefined)).toEqual([]);
  });

  it("should return an empty array when input is an empty array", () => {
    const enabledNodeIds = new Set(["A", "B"]);
    expect(convertEdges(enabledNodeIds, [])).toEqual([]);
  });

  it("should correctly convert a single edge", () => {
    const input: Edge[] = [{ id: "1", source: "A", target: "B" }];
    const enabledNodeIds = new Set(["A", "B"]);
    const result = convertEdges(enabledNodeIds, input);

    expect(result).toHaveLength(1);
    expect(result[0]).toMatchObject({
      from: "A",
      to: "B",
      fromPort: "default",
      toPort: "default",
    });
    expect(typeof result[0].id).toBe("string");
    expect(result[0].id).not.toBe("");
  });

  it("should correctly convert multiple edges with unique IDs", () => {
    const input: Edge[] = [
      { id: "1", source: "A", target: "B" },
      {
        id: "2",
        source: "B",
        target: "C",
        sourceHandle: "output",
        targetHandle: "input",
      },
    ];
    const enabledNodeIds = new Set(["A", "B", "C"]);
    const result = convertEdges(enabledNodeIds, input);

    expect(result).toHaveLength(2);
    expect(result[0]).toMatchObject({
      from: "A",
      to: "B",
      fromPort: "default",
      toPort: "default",
    });
    expect(result[1]).toMatchObject({
      from: "B",
      to: "C",
      fromPort: "output",
      toPort: "input",
    });
    // Verify IDs are unique
    expect(result[0].id).not.toBe(result[1].id);
  });

  it("should use default ports when sourceHandle and targetHandle are not provided", () => {
    const input: Edge[] = [{ id: "1", source: "A", target: "B" }];
    const enabledNodeIds = new Set(["A", "B"]);
    const result = convertEdges(enabledNodeIds, input);

    expect(result).toHaveLength(1);
    expect(result[0]).toMatchObject({
      from: "A",
      to: "B",
      fromPort: "default",
      toPort: "default",
    });
  });

  it("should use provided sourceHandle and targetHandle when available", () => {
    const input: Edge[] = [
      {
        id: "1",
        source: "A",
        target: "B",
        sourceHandle: "output1",
        targetHandle: "input1",
      },
    ];
    const enabledNodeIds = new Set(["A", "B"]);
    const result = convertEdges(enabledNodeIds, input);

    expect(result).toHaveLength(1);
    expect(result[0]).toMatchObject({
      from: "A",
      to: "B",
      fromPort: "output1",
      toPort: "input1",
    });
  });

  it("should handle a mix of edges with and without custom handles and generate unique IDs", () => {
    const input: Edge[] = [
      { id: "1", source: "A", target: "B" },
      {
        id: "2",
        source: "B",
        target: "C",
        sourceHandle: "output",
        targetHandle: "input",
      },
      { id: "3", source: "C", target: "D", sourceHandle: "output" },
      { id: "4", source: "D", target: "E", targetHandle: "input" },
    ];
    const enabledNodeIds = new Set(["A", "B", "C", "D", "E"]);
    const result = convertEdges(enabledNodeIds, input);

    expect(result).toHaveLength(4);
    expect(result[0]).toMatchObject({
      from: "A",
      to: "B",
      fromPort: "default",
      toPort: "default",
    });
    expect(result[1]).toMatchObject({
      from: "B",
      to: "C",
      fromPort: "output",
      toPort: "input",
    });
    expect(result[2]).toMatchObject({
      from: "C",
      to: "D",
      fromPort: "output",
      toPort: "default",
    });
    expect(result[3]).toMatchObject({
      from: "D",
      to: "E",
      fromPort: "default",
      toPort: "input",
    });

    // Verify all IDs are unique
    const ids = result.map((edge) => edge.id);
    const uniqueIds = new Set(ids);
    expect(uniqueIds.size).toBe(result.length);
  });
});
