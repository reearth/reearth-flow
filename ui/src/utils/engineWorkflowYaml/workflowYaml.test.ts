import { describe, it, expect, vi, beforeEach } from "vitest";
import YAML from "yaml";

import type { Workflow } from "@flow/types";

import { consolidateWorkflows } from "./consolidateWorkflows";
import { createWorkflowsYaml } from "./workflowYaml";

vi.mock("./consolidateWorkflows", () => ({
  consolidateWorkflows: vi.fn(),
}));

vi.mock("yaml", () => ({
  default: {
    stringify: vi.fn(),
  },
}));

describe("createWorkflowsYaml", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should return undefined when workflows is undefined", () => {
    const result = createWorkflowsYaml(undefined);
    expect(result).toBeUndefined();
    expect(consolidateWorkflows).not.toHaveBeenCalled();
    expect(YAML.stringify).not.toHaveBeenCalled();
  });

  it("should correctly create YAML from workflows", () => {
    const mockWorkflows: Workflow[] = [
      { id: "workflow1", name: "Workflow 1", nodes: [], edges: [] },
    ];

    const mockConsolidatedWorkflow = {
      id: "consolidated-id",
      name: "My project's workflow",
      entryGraphId: "workflow1",
      graphs: [{ id: "workflow1", name: "Workflow 1", nodes: [], edges: [] }],
    };

    const mockYamlString = "mockYamlString";

    (consolidateWorkflows as any).mockReturnValue(mockConsolidatedWorkflow);
    (YAML.stringify as any).mockReturnValue(mockYamlString);

    const result = createWorkflowsYaml(mockWorkflows);

    expect(result).toEqual({
      workflowId: "consolidated-id",
      yamlWorkflow: mockYamlString,
    });

    expect(consolidateWorkflows).toHaveBeenCalledWith(mockWorkflows);
    expect(YAML.stringify).toHaveBeenCalledWith(mockConsolidatedWorkflow);
  });

  it("should handle empty workflows array", () => {
    const mockConsolidatedWorkflow = {
      id: "consolidated-id",
      name: "My project's workflow",
      entryGraphId: undefined,
      graphs: [],
    };

    const mockYamlString = "mockYamlString";

    (consolidateWorkflows as any).mockReturnValue(mockConsolidatedWorkflow);
    (YAML.stringify as any).mockReturnValue(mockYamlString);

    const result = createWorkflowsYaml([]);

    expect(result).toEqual({
      workflowId: "consolidated-id",
      yamlWorkflow: mockYamlString,
    });

    expect(consolidateWorkflows).toHaveBeenCalledWith([]);
    expect(YAML.stringify).toHaveBeenCalledWith(mockConsolidatedWorkflow);
  });

  it("should handle YAML.stringify errors", () => {
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
    (YAML.stringify as any).mockImplementation(() => {
      throw new Error("YAML stringify error");
    });

    expect(() => createWorkflowsYaml(mockWorkflows)).toThrow(
      "YAML stringify error",
    );

    expect(consolidateWorkflows).toHaveBeenCalledWith(mockWorkflows);
    expect(YAML.stringify).toHaveBeenCalledWith(mockConsolidatedWorkflow);
  });
});
