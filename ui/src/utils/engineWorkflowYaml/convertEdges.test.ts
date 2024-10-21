import { describe, it, expect } from "vitest";

import type { Edge } from "@flow/types";

import { convertEdges } from "./convertEdges";

describe("convertEdges", () => {
  it("should return an empty array when input is undefined", () => {
    expect(convertEdges(undefined)).toEqual([]);
  });

  it("should return an empty array when input is an empty array", () => {
    expect(convertEdges([])).toEqual([]);
  });

  it("should correctly convert a single edge", () => {
    const input: Edge[] = [{ id: "1", source: "A", target: "B" }];
    const expected = [
      { id: "1", from: "A", to: "B", fromPort: "default", toPort: "default" },
    ];
    expect(convertEdges(input)).toEqual(expected);
  });

  it("should correctly convert multiple edges", () => {
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
    const expected = [
      { id: "1", from: "A", to: "B", fromPort: "default", toPort: "default" },
      { id: "2", from: "B", to: "C", fromPort: "output", toPort: "input" },
    ];
    expect(convertEdges(input)).toEqual(expected);
  });

  it("should use default ports when sourceHandle and targetHandle are not provided", () => {
    const input: Edge[] = [{ id: "1", source: "A", target: "B" }];
    const expected = [
      { id: "1", from: "A", to: "B", fromPort: "default", toPort: "default" },
    ];
    expect(convertEdges(input)).toEqual(expected);
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
    const expected = [
      { id: "1", from: "A", to: "B", fromPort: "output1", toPort: "input1" },
    ];
    expect(convertEdges(input)).toEqual(expected);
  });

  it("should handle a mix of edges with and without custom handles", () => {
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
    const expected = [
      { id: "1", from: "A", to: "B", fromPort: "default", toPort: "default" },
      { id: "2", from: "B", to: "C", fromPort: "output", toPort: "input" },
      { id: "3", from: "C", to: "D", fromPort: "output", toPort: "default" },
      { id: "4", from: "D", to: "E", fromPort: "default", toPort: "input" },
    ];
    expect(convertEdges(input)).toEqual(expected);
  });
});
