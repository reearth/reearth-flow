import dagre from "@dagrejs/dagre";

import { DEFAULT_NODE_SIZE } from "@flow/global-constants";
import { Algorithm, Direction, Edge, Node } from "@flow/types";

export type DagreDirection = "TB" | "LR";

const dagreGraph = new dagre.graphlib.Graph().setDefaultEdgeLabel(() => ({}));

export const autoLayout = (
  algorithm: Algorithm = "dagre",
  direction: Direction = "Horizontal",
  nodes: Node[],
  edges: Edge[],
) => {
  // Currently only supporting dagre
  if (algorithm === "dagre") {
    const dagreDirection = direction === "Horizontal" ? "LR" : "TB";
    // const isHorizontal = direction === "LR";
    dagreGraph.setGraph({ rankdir: dagreDirection });

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
        // targetPosition: isHorizontal ? "left" : "top",
        // sourcePosition: isHorizontal ? "right" : "bottom",
        // We are shifting the dagre node position (anchor=center center) to the top left
        // so it matches the React Flow node anchor point (top left).
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
