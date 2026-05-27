import dagre from "@dagrejs/dagre";

import {
  DEFAULT_LAYOUT_X_SPACING,
  DEFAULT_LAYOUT_Y_SPACING,
  DEFAULT_NODE_SIZE,
} from "@flow/global-constants";
import { Algorithm, Direction, Edge, Node } from "@flow/types";

export type DagreDirection = "TB" | "LR";

const dagreGraph = new dagre.graphlib.Graph().setDefaultEdgeLabel(() => ({}));

export const autoLayout = (
  algorithm: Algorithm = "dagre",
  direction: Direction = "Horizontal",
  nodes: Node[],
  edges: Edge[],
) => {
  if (algorithm === "dagre") {
    const isHorizontal = direction === "Horizontal";
    // In LR layout: ranksep = gap between columns (x), nodesep = gap between rows (y)
    // In TB layout: ranksep = gap between rows (y), nodesep = gap between columns (x)
    dagreGraph.setGraph({
      rankdir: isHorizontal ? "LR" : "TB",
      ranksep: isHorizontal
        ? DEFAULT_LAYOUT_X_SPACING
        : DEFAULT_LAYOUT_Y_SPACING,
      nodesep: isHorizontal
        ? DEFAULT_LAYOUT_Y_SPACING
        : DEFAULT_LAYOUT_X_SPACING,
    });

    nodes.forEach((node) => {
      dagreGraph.setNode(node.id, {
        width: DEFAULT_NODE_SIZE.width,
        height: DEFAULT_NODE_SIZE.height,
      });
    });

    edges.forEach((edge) => {
      dagreGraph.setEdge(edge.source, edge.target);
    });

    dagre.layout(dagreGraph);

    const newNodes: Node[] = nodes.map((node) => {
      const nodeWithPosition = dagreGraph.node(node.id);
      const newNode: Node = {
        ...node,
        // Shift dagre center anchor to React Flow top-left anchor
        position: {
          x: nodeWithPosition.x - DEFAULT_NODE_SIZE.width / 2,
          y: nodeWithPosition.y - DEFAULT_NODE_SIZE.height / 2,
        },
      };

      return newNode;
    });

    return { nodes: newNodes, edges };
  }

  return { nodes, edges };
};
