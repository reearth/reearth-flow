import { describe, it, expect, vi, beforeEach } from "vitest";

import { DEFAULT_NODE_SIZE } from "@flow/global-constants";
import { fetcher } from "@flow/lib/fetch/transformers/useFetch";
import type { Action, EngineReadyNode } from "@flow/types";

import { convertNodes } from "./convertNodes";

const API_URL = process.env.API;

// Mock the external dependencies
vi.mock("@flow/config", () => ({
  config: () => ({
    api: API_URL,
  }),
}));

vi.mock("@flow/lib/fetch/transformers/useFetch", () => ({
  fetcher: vi.fn(),
}));

describe("convertNodes", () => {
  const mockAction: Action = {
    name: "Test Action",
    description: "Test action description",
    categories: ["Test"],
    builtin: true,
    parameter: {},
    type: "testAction",
    inputPorts: ["input1"],
    outputPorts: ["output1"],
  };

  const mockPseudoPorts = {
    pseudoInputs: [{ id: "pseudo1", name: "Pseudo Input 1" }],
    pseudoOutputs: [{ id: "pseudo2", name: "Pseudo Output 1" }],
  };

  const mockGetSubworkflowPseudoPorts = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
    (fetcher as any).mockReset();
    mockGetSubworkflowPseudoPorts.mockReset();
  });

  it("should return empty array when input is undefined", async () => {
    const result = await convertNodes(
      undefined as unknown as EngineReadyNode[],
      mockGetSubworkflowPseudoPorts,
    );
    expect(result).toEqual([]);
  });

  it("should convert a regular action node correctly", async () => {
    const engineNode: EngineReadyNode = {
      id: "node1",
      name: "Test Node",
      type: "regular",
      action: "test-action",
    };

    (fetcher as any).mockResolvedValueOnce(mockAction);

    const result = await convertNodes(
      [engineNode],
      mockGetSubworkflowPseudoPorts,
    );

    expect(fetcher).toHaveBeenCalledWith(`${API_URL}/actions/test-action`);
    expect(result).toHaveLength(1);
    expect(result[0]).toEqual({
      id: "node1",
      type: "testAction",
      position: { x: 0, y: 0 },
      measured: DEFAULT_NODE_SIZE,
      data: {
        officialName: "test-action",
        params: undefined,
        customizations: {
          customName: "Test Node",
        },
        inputs: mockAction.inputPorts,
        outputs: mockAction.outputPorts,
      },
    });
  });

  it("should convert a subworkflow node correctly", async () => {
    const engineNode: EngineReadyNode = {
      id: "sub1",
      name: "Sub Workflow",
      type: "subGraph",
      subGraphId: "workflow1",
    };

    mockGetSubworkflowPseudoPorts.mockReturnValueOnce(mockPseudoPorts);

    const result = await convertNodes(
      [engineNode],
      mockGetSubworkflowPseudoPorts,
    );

    expect(mockGetSubworkflowPseudoPorts).toHaveBeenCalledWith("workflow1");
    expect(result).toHaveLength(1);
    expect(result[0]).toEqual({
      id: "sub1",
      type: "subworkflow",
      position: { x: 0, y: 0 },
      measured: DEFAULT_NODE_SIZE,
      data: {
        officialName: "Subworkflow",
        params: undefined,
        customizations: {
          customName: "Sub Workflow",
        },
        subworkflowId: "workflow1",
        pseudoInputs: mockPseudoPorts.pseudoInputs,
        pseudoOutputs: mockPseudoPorts.pseudoOutputs,
      },
    });
  });

  it("should handle missing subworkflow pseudo ports", async () => {
    const engineNode: EngineReadyNode = {
      id: "sub1",
      name: "Sub Workflow",
      type: "subGraph",
      subGraphId: "workflow1",
    };

    mockGetSubworkflowPseudoPorts.mockReturnValueOnce(undefined);

    const result = await convertNodes(
      [engineNode],
      mockGetSubworkflowPseudoPorts,
    );

    expect(result[0]?.data).not.toHaveProperty("pseudoInputs");
    expect(result[0]?.data).not.toHaveProperty("pseudoOutputs");
  });

  it("should handle nodes with parameters", async () => {
    const engineNode: EngineReadyNode = {
      id: "node1",
      name: "Test Node",
      type: "regular",
      action: "test-action",
      with: { param1: "value1", param2: "value2" },
    };

    (fetcher as any).mockResolvedValueOnce(mockAction);

    const result = await convertNodes(
      [engineNode],
      mockGetSubworkflowPseudoPorts,
    );

    expect(result[0]?.data.params).toEqual({
      param1: "value1",
      param2: "value2",
    });
  });

  it("should handle multiple nodes of different types", async () => {
    const engineNodes: EngineReadyNode[] = [
      {
        id: "node1",
        name: "Action Node",
        type: "action",
        action: "FeatureCreator",
      },
      {
        id: "sub1",
        name: "Sub Workflow",
        type: "subGraph",
        subGraphId: "workflow1",
      },
    ];

    (fetcher as any).mockResolvedValueOnce(mockAction);
    mockGetSubworkflowPseudoPorts.mockReturnValueOnce(mockPseudoPorts);

    const result = await convertNodes(
      engineNodes,
      mockGetSubworkflowPseudoPorts,
    );

    expect(result).toHaveLength(2);
    expect(result[0]?.type).toBe("testAction");
    expect(result[1]?.type).toBe("subworkflow");
  });
});
