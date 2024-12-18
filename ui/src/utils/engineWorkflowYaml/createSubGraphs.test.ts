import { describe, expect, vi } from "vitest";

import type {
  Workflow,
  Node,
  Edge,
  EngineReadyNode,
  EngineReadyEdge,
  EngineReadyGraph,
} from "@flow/types";

import { convertEdges } from "./convertEdges";
import { convertNodes } from "./convertNodes";
import { createSubGraphs } from "./createSubGraphs";

vi.mock("./convertEdges", () => ({
  convertEdges: vi.fn(),
}));

vi.mock("./convertNodes", () => ({
  convertNodes: vi.fn(),
}));

describe("createSubGraphs", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  test("should return an empty array when input is an empty array", () => {
    expect(createSubGraphs([])).toEqual([]);
  });

  test("should correctly create sub-graphs for a single workflow", () => {
    const mockNodes: Node[] = [
      {
        id: "1",
        type: "transformer",
        position: { x: 22, y: 22 },
        data: { officialName: "Node 1" },
      },
      {
        id: "2",
        type: "transformer",
        position: { x: 22, y: 22 },
        data: { officialName: "Node 2" },
      },
    ];
    const mockEdges: Edge[] = [
      { id: "edge1", source: "node1", target: "node2" },
    ];
    const mockWorkflow: Workflow = {
      id: "workflow1",
      name: "Test Workflow",
      nodes: mockNodes,
      edges: mockEdges,
    };

    const mockConvertedNodes: EngineReadyNode[] = [
      { id: "node1", name: "Node 1", type: "test" },
      { id: "node2", name: "Node 2", type: "test" },
    ];
    const mockConvertedEdges: EngineReadyEdge[] = [
      {
        id: "edge1",
        from: "node1",
        to: "node2",
        fromPort: "default",
        toPort: "default",
      },
    ];

    (convertNodes as any).mockReturnValue(mockConvertedNodes);
    (convertEdges as any).mockReturnValue(mockConvertedEdges);

    const result = createSubGraphs([mockWorkflow]);

    expect(result).toEqual([
      {
        id: "workflow1",
        name: "Test Workflow",
        nodes: mockConvertedNodes,
        edges: mockConvertedEdges,
      },
    ]);

    expect(convertNodes).toHaveBeenCalledWith(mockNodes);
    expect(convertEdges).toHaveBeenCalledWith(mockEdges);
  });

  test("should correctly create sub-graphs for multiple workflows", () => {
    const mockWorkflows: Workflow[] = [
      {
        id: "workflow1",
        name: "Workflow 1",
        nodes: [
          {
            id: "1",
            type: "transformer",
            position: { x: 22, y: 22 },
            data: { officialName: "Node 1" },
          },
        ],
        edges: [{ id: "edge1", source: "node1", target: "node2" }],
      },
      {
        id: "workflow2",
        name: "Workflow 2",
        nodes: [
          {
            id: "2",
            type: "transformer",
            position: { x: 22, y: 22 },
            data: { officialName: "Node 2" },
          },
        ],
        edges: [{ id: "edge2", source: "node2", target: "node3" }],
      },
    ];

    (convertNodes as any).mockImplementation((nodes: Node[]) =>
      nodes.map(
        (node): EngineReadyNode => ({
          id: node.id,
          name: node.data.officialName ?? "undefined",
          type: node.type ?? "undefined",
        }),
      ),
    );
    (convertEdges as any).mockImplementation((edges: Edge[]) =>
      edges.map((edge) => ({
        id: edge.id,
        from: edge.source,
        to: edge.target,
        fromPort: "default",
        toPort: "default",
      })),
    );

    const result = createSubGraphs(mockWorkflows);

    const expected: EngineReadyGraph[] = [
      {
        id: "workflow1",
        name: "Workflow 1",
        nodes: [{ id: "1", name: "Node 1", type: "transformer" }],
        edges: [
          {
            id: "edge1",
            from: "node1",
            to: "node2",
            fromPort: "default",
            toPort: "default",
          },
        ],
      },
      {
        id: "workflow2",
        name: "Workflow 2",
        nodes: [{ id: "2", name: "Node 2", type: "transformer" }],
        edges: [
          {
            id: "edge2",
            from: "node2",
            to: "node3",
            fromPort: "default",
            toPort: "default",
          },
        ],
      },
    ];

    expect(result).toEqual(expected);

    expect(convertNodes).toHaveBeenCalledTimes(2);
    expect(convertEdges).toHaveBeenCalledTimes(2);
  });

  it('should use "undefined-graph" as name when workflow name is not provided', () => {
    const mockWorkflow: Workflow = {
      id: "workflow1",
      nodes: [],
      edges: [],
    };

    (convertNodes as any).mockReturnValue([]);
    (convertEdges as any).mockReturnValue([]);

    const result = createSubGraphs([mockWorkflow]);

    expect(result).toEqual([
      {
        id: "workflow1",
        name: "undefined-graph",
        nodes: [],
        edges: [],
      },
    ]);
  });
});
