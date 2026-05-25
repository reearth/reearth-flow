import dagre from "@dagrejs/dagre";

import { DEFAULT_GRID_SIZE, DEFAULT_NODE_SIZE } from "@flow/global-constants";
import { Direction, Edge, Node } from "@flow/types";

const nodeWidth = (node: Node) =>
  node.measured?.width ?? DEFAULT_NODE_SIZE.width;
const nodeHeight = (node: Node) =>
  node.measured?.height ?? DEFAULT_NODE_SIZE.height;

const snapToGrid = (v: number) =>
  Math.round(v / DEFAULT_GRID_SIZE) * DEFAULT_GRID_SIZE;

export const DEFAULT_LAYOUT_SPACING = { x: 80, y: 50 };

export const autoLayout = async (
  direction: Direction = "Horizontal",
  nodes: Node[],
  edges: Edge[],
  xSpacing: number = DEFAULT_LAYOUT_SPACING.x,
  ySpacing: number = DEFAULT_LAYOUT_SPACING.y,
): Promise<{ nodes: Node[]; edges: Edge[] }> => {
  const graph = new dagre.graphlib.Graph().setDefaultEdgeLabel(() => ({}));
  const isHorizontal = direction === "Horizontal";
  graph.setGraph({
    rankdir: isHorizontal ? "LR" : "TB",
    ranksep: isHorizontal ? xSpacing : ySpacing,
    nodesep: isHorizontal ? ySpacing : xSpacing,
  });

  nodes.forEach((node) => {
    graph.setNode(node.id, {
      width: nodeWidth(node),
      height: nodeHeight(node),
    });
  });

  edges.forEach((edge) => {
    graph.setEdge(edge.source, edge.target);
  });

  dagre.layout(graph);

  return {
    nodes: nodes.map((node) => {
      const { x, y } = graph.node(node.id);
      return {
        ...node,
        position: {
          x: snapToGrid(x - nodeWidth(node) / 2),
          y: snapToGrid(y - nodeHeight(node) / 2),
        },
      };
    }),
    edges,
  };
};
