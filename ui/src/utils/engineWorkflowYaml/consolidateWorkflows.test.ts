import { describe, it, expect, vi, beforeEach } from "vitest";

import type { Workflow, EngineReadyGraph } from "@flow/types";

import { randomID } from "../randomID";

import { consolidateWorkflows } from "./consolidateWorkflows";
import { createSubGraphs } from "./createSubGraphs";

vi.mock("./createSubGraphs", () => ({
  createSubGraphs: vi.fn(),
}));

vi.mock("../randomID", () => ({
  randomID: vi.fn(),
}));

describe("consolidateWorkflows", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should correctly consolidate workflows with a main workflow", () => {
    const mockWorkflows: Workflow[] = [
      { id: "main", name: "Main Workflow", nodes: [], edges: [] },
      { id: "sub1", name: "Sub Workflow 1", nodes: [], edges: [] },
    ];

    const mockSubGraphs: EngineReadyGraph[] = [
      { id: "main", name: "Main Workflow", nodes: [], edges: [] },
      { id: "sub1", name: "Sub Workflow 1", nodes: [], edges: [] },
    ];

    (createSubGraphs as any).mockReturnValue(mockSubGraphs);
    (vi.mocked(randomID) as any).mockReturnValue("random-id-123");

    const result = consolidateWorkflows("somename", mockWorkflows);

    expect(result).toEqual({
      id: "random-id-123",
      name: "somename",
      entryGraphId: "main",
      graphs: mockSubGraphs,
    });

    expect(createSubGraphs).toHaveBeenCalledWith(mockWorkflows);
    expect(randomID).toHaveBeenCalled();
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
    (vi.mocked(randomID) as any).mockReturnValue("random-id-456");

    const result = consolidateWorkflows("somename", mockWorkflows);

    expect(result).toEqual(undefined);

    expect(createSubGraphs).not.toHaveBeenCalledWith(mockWorkflows);
    expect(randomID).not.toHaveBeenCalled();
  });

  it("should return undefined when workflows array is empty (since there is no main/entry point)", () => {
    const mockWorkflows: Workflow[] = [];

    (createSubGraphs as any).mockReturnValue([]);
    (vi.mocked(randomID) as any).mockReturnValue("random-id-789");

    const result = consolidateWorkflows("somename", mockWorkflows);

    expect(result).toEqual(undefined);
    expect(createSubGraphs).not.toHaveBeenCalledWith(mockWorkflows);
    expect(randomID).not.toHaveBeenCalled();
  });
});
