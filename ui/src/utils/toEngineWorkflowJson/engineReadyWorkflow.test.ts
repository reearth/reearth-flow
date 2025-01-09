import { describe, it, expect, vi, beforeEach } from "vitest";

import type { Workflow } from "@flow/types";

import { consolidateWorkflows } from "./consolidateWorkflows";
import { createEngineReadyWorkflow } from "./engineReadyWorkflow";

vi.mock("./consolidateWorkflows", () => ({
  consolidateWorkflows: vi.fn(),
}));

describe("createEngineReadyWorkflow", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should return undefined when workflows is undefined", () => {
    const result = createEngineReadyWorkflow(undefined);
    expect(result).toBeUndefined();
    expect(consolidateWorkflows).not.toHaveBeenCalled();
  });

  it("should correctly create engine ready workflow", () => {
    const mockWorkflows: Workflow[] = [
      { id: "workflow1", name: "Workflow 1", nodes: [], edges: [] },
    ];
    const mockConsolidatedWorkflow = {
      id: "consolidated-id",
      name: "My project's workflow",
      entryGraphId: "workflow1",
      graphs: [{ id: "workflow1", name: "Workflow 1", nodes: [], edges: [] }],
    };

    (consolidateWorkflows as any).mockReturnValue(mockConsolidatedWorkflow);

    const result = createEngineReadyWorkflow("somename", mockWorkflows);
    expect(result).toEqual(mockConsolidatedWorkflow);
    expect(consolidateWorkflows).toHaveBeenCalledWith(
      "somename-workflow",
      mockWorkflows,
    );
  });

  it("should handle empty workflows array", () => {
    const mockConsolidatedWorkflow = {
      id: "consolidated-id",
      name: "My project's workflow",
      entryGraphId: undefined,
      graphs: [],
    };

    (consolidateWorkflows as any).mockReturnValue(mockConsolidatedWorkflow);

    const result = createEngineReadyWorkflow("somename", []);
    expect(result).toEqual(mockConsolidatedWorkflow);
    expect(consolidateWorkflows).toHaveBeenCalledWith("somename-workflow", []);
  });

  it("should return undefined when consolidateWorkflows returns undefined", () => {
    const mockWorkflows: Workflow[] = [
      { id: "workflow1", name: "Workflow 1", nodes: [], edges: [] },
    ];

    (consolidateWorkflows as any).mockReturnValue(undefined);

    const result = createEngineReadyWorkflow("somename", mockWorkflows);
    expect(result).toBeUndefined();
    expect(consolidateWorkflows).toHaveBeenCalledWith(
      "somename-workflow",
      mockWorkflows,
    );
  });
});
