import { describe, it, expect, vi, beforeEach } from "vitest";

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
    const workflow1Id = generateUUID();
    const workflow2Id = generateUUID();
    const mockWorkflows: Workflow[] = [
      {
        id: workflow1Id,
        name: "Main Workflow",
        isMain: true,
        nodes: [],
        edges: [],
      },
      { id: workflow2Id, name: "Sub Workflow 1", nodes: [], edges: [] },
    ];

    // Generate predictable UUIDs
    let uuidCounter = 0;
    vi.mocked(generateUUID).mockImplementation(() => `uuid-${++uuidCounter}`);

    const mockSubGraphs: EngineReadyGraph[] = [
      {
        id: workflow1Id,
        name: "Main Workflow",
        nodes: [],
        edges: [],
      },
      { id: "sub1", name: "Sub Workflow 1", nodes: [], edges: [] },
    ];

    vi.mocked(createSubGraphs).mockReturnValue(mockSubGraphs);

    const result = consolidateWorkflows("somename", mockWorkflows);

    expect(result?.name).toEqual("somename");
    expect(result?.entryGraphId).toEqual(workflow1Id);
    expect(result?.graphs).toEqual(mockSubGraphs);

    expect(createSubGraphs).toHaveBeenCalledWith(mockWorkflows);
    expect(generateUUID).toHaveBeenCalledTimes(3);
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
