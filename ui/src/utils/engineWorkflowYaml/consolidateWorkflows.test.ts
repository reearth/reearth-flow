import { describe, it, expect, vi, beforeEach } from "vitest";

import { DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";
import type { Workflow, EngineReadyGraph } from "@flow/types";

import { generateUUID } from "../generateUUID";

import { consolidateWorkflows } from "./consolidateWorkflows";
import { createSubGraphs } from "./createSubGraphs";

vi.mock("./createSubGraphs", () => ({
  createSubGraphs: vi.fn(),
}));

vi.mock("../generateUUID", () => ({
  generateUUID: vi.fn(),
}));

describe("consolidateWorkflows", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should correctly consolidate workflows with a main workflow", () => {
    const mockWorkflows: Workflow[] = [
      {
        id: DEFAULT_ENTRY_GRAPH_ID,
        name: "Main Workflow",
        nodes: [],
        edges: [],
      },
      { id: "sub1", name: "Sub Workflow 1", nodes: [], edges: [] },
    ];

    // Generate predictable UUIDs
    let uuidCounter = 0;
    vi.mocked(generateUUID).mockImplementation(() => `uuid-${++uuidCounter}`);

    // Create expected workflows with the new entry ID
    const expectedConvertedWorkflows = [
      {
        id: "uuid-1", // This replaces DEFAULT_ENTRY_GRAPH_ID
        name: "Main Workflow",
        nodes: [],
        edges: [],
      },
      { id: "sub1", name: "Sub Workflow 1", nodes: [], edges: [] },
    ];

    const mockSubGraphs: EngineReadyGraph[] = [
      {
        id: "uuid-1",
        name: "Main Workflow",
        nodes: [],
        edges: [],
      },
      { id: "sub1", name: "Sub Workflow 1", nodes: [], edges: [] },
    ];

    vi.mocked(createSubGraphs).mockReturnValue(mockSubGraphs);

    const result = consolidateWorkflows("somename", mockWorkflows);

    expect(result).toEqual({
      id: "uuid-2",
      name: "somename",
      entryGraphId: "uuid-1",
      graphs: mockSubGraphs,
    });

    expect(createSubGraphs).toHaveBeenCalledWith(expectedConvertedWorkflows);
    expect(generateUUID).toHaveBeenCalledTimes(2);
  });

  it("should return undefined when no main workflow exists", () => {
    const mockWorkflows: Workflow[] = [
      { id: "sub1", name: "Sub Workflow 1", nodes: [], edges: [] },
      { id: "sub2", name: "Sub Workflow 2", nodes: [], edges: [] },
    ];

    const result = consolidateWorkflows("somename", mockWorkflows);

    expect(result).toBeUndefined();
    expect(createSubGraphs).not.toHaveBeenCalled();
    expect(generateUUID).not.toHaveBeenCalled();
  });

  it("should return undefined when workflows array is empty", () => {
    const mockWorkflows: Workflow[] = [];

    const result = consolidateWorkflows("somename", mockWorkflows);

    expect(result).toBeUndefined();
    expect(createSubGraphs).not.toHaveBeenCalled();
    expect(generateUUID).not.toHaveBeenCalled();
  });
});
