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

    const mockSubGraphs: EngineReadyGraph[] = [
      {
        id: DEFAULT_ENTRY_GRAPH_ID,
        name: "Main Workflow",
        nodes: [],
        edges: [],
      },
      { id: "sub1", name: "Sub Workflow 1", nodes: [], edges: [] },
    ];

    (createSubGraphs as any).mockReturnValue(mockSubGraphs);
    (vi.mocked(generateUUID) as any).mockReturnValue("random-id-123");

    const result = consolidateWorkflows("somename", mockWorkflows);

    expect(result).toEqual({
      id: "random-id-123",
      name: "somename",
      entryGraphId: DEFAULT_ENTRY_GRAPH_ID,
      graphs: mockSubGraphs,
    });

    expect(createSubGraphs).toHaveBeenCalledWith(mockWorkflows);
    expect(generateUUID).toHaveBeenCalled();
  });

  it("should correctly consolidate workflows without a main workflow", () => {
    const mockWorkflows: Workflow[] = [
      { id: "sub1", name: "Sub Workflow 1", nodes: [], edges: [] },
      { id: "sub2", name: "Sub Workflow 2", nodes: [], edges: [] },
    ];

    const mockSubGraphs: EngineReadyGraph[] = [
      { id: "sub1", name: "Sub Workflow 1", nodes: [], edges: [] },
      { id: "sub2", name: "Sub Workflow 2", nodes: [], edges: [] },
    ];

    (createSubGraphs as any).mockReturnValue(mockSubGraphs);
    (vi.mocked(generateUUID) as any).mockReturnValue("random-id-456");

    const result = consolidateWorkflows("somename", mockWorkflows);

    expect(result).toEqual(undefined);

    expect(createSubGraphs).not.toHaveBeenCalledWith(mockWorkflows);
    expect(generateUUID).not.toHaveBeenCalled();
  });

  it("should return undefined when workflows array is empty (since there is no main/entry point)", () => {
    const mockWorkflows: Workflow[] = [];

    (createSubGraphs as any).mockReturnValue([]);
    (vi.mocked(generateUUID) as any).mockReturnValue("random-id-789");

    const result = consolidateWorkflows("somename", mockWorkflows);

    expect(result).toEqual(undefined);
    expect(createSubGraphs).not.toHaveBeenCalledWith(mockWorkflows);
    expect(generateUUID).not.toHaveBeenCalled();
  });
});
