import dagre from "@dagrejs/dagre";
import { stratify, tree } from "d3-hierarchy";
import type { HierarchyPointNode } from "d3-hierarchy";
import type { ELK as ELKInstance } from "elkjs/lib/elk-api";
// The bundled build avoids the `web-worker` Node.js package that breaks Vite production builds.
// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-ignore
import ELKBundled from "elkjs/lib/elk.bundled.js";

import { DEFAULT_NODE_SIZE } from "@flow/global-constants";
import { Algorithm, Direction, Edge, Node } from "@flow/types";

const elk = new ELKBundled() as ELKInstance;

const nodeWidth = (node: Node) =>
  node.measured?.width ?? DEFAULT_NODE_SIZE.width;
const nodeHeight = (node: Node) =>
  node.measured?.height ?? DEFAULT_NODE_SIZE.height;

const dagreLayout = (
  direction: Direction,
  nodes: Node[],
  edges: Edge[],
): { nodes: Node[]; edges: Edge[] } => {
  const graph = new dagre.graphlib.Graph().setDefaultEdgeLabel(() => ({}));
  graph.setGraph({
    rankdir: direction === "Horizontal" ? "LR" : "TB",
    ranksep: 80,
    nodesep: 40,
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
          x: x - nodeWidth(node) / 2,
          y: y - nodeHeight(node) / 2,
        },
      };
    }),
    edges,
  };
};

const elkLayout = async (
  direction: Direction,
  nodes: Node[],
  edges: Edge[],
): Promise<{ nodes: Node[]; edges: Edge[] }> => {
  const elkGraph = {
    id: "root",
    layoutOptions: {
      "elk.algorithm": "layered",
      "elk.direction": direction === "Horizontal" ? "RIGHT" : "DOWN",
      "elk.spacing.nodeNode": "50",
      "elk.layered.spacing.nodeNodeBetweenLayers": "80",
    },
    children: nodes.map((node) => ({
      id: node.id,
      width: nodeWidth(node),
      height: nodeHeight(node),
    })),
    edges: edges.map((edge) => ({
      id: edge.id,
      sources: [edge.source],
      targets: [edge.target],
    })),
  };

  const result = await elk.layout(elkGraph);

  return {
    nodes: nodes.map((node) => {
      const child = result.children?.find((c) => c.id === node.id);
      if (!child) return node;
      return {
        ...node,
        position: {
          x: child.x ?? node.position.x,
          y: child.y ?? node.position.y,
        },
      };
    }),
    edges,
  };
};

type D3Datum = { id: string; parentId: string | null };

const d3Layout = (
  direction: Direction,
  nodes: Node[],
  edges: Edge[],
): { nodes: Node[]; edges: Edge[] } => {
  const targetIds = new Set(edges.map((e) => e.target));
  const roots = nodes.filter((n) => !targetIds.has(n.id));

  // D3 tree requires exactly one root
  if (roots.length !== 1) return { nodes, edges };

  // D3 tree requires no node to have more than one parent
  const incomingCount = new Map<string, number>();
  edges.forEach((e) =>
    incomingCount.set(e.target, (incomingCount.get(e.target) ?? 0) + 1),
  );
  if ([...incomingCount.values()].some((count) => count > 1))
    return { nodes, edges };

  const parentMap = new Map<string, string>();
  edges.forEach((e) => parentMap.set(e.target, e.source));

  // Use the largest node dimensions to guarantee no overlap across mixed node types
  const maxWidth = Math.max(...nodes.map(nodeWidth));
  const maxHeight = Math.max(...nodes.map(nodeHeight));

  try {
    const hierarchy = stratify<D3Datum>()
      .id((d) => d.id)
      .parentId((d) => d.parentId)(
      nodes.map((n) => ({
        id: n.id,
        parentId: parentMap.get(n.id) ?? null,
      })),
    );

    // nodeSize: [x-separation (perpendicular to depth), y-separation (depth)]
    const layout = tree<D3Datum>().nodeSize(
      direction === "Horizontal"
        ? [maxHeight + 40, maxWidth + 80]
        : [maxWidth + 40, maxHeight + 80],
    );

    layout(hierarchy);

    const positionMap = new Map<string, { x: number; y: number }>();
    // tree() assigns x/y on all nodes; cast from HierarchyNode to HierarchyPointNode
    (hierarchy as unknown as HierarchyPointNode<D3Datum>).each((node) => {
      if (!node.id) return;
      positionMap.set(node.id, {
        // For horizontal (LR): depth (node.y) → x, perpendicular (node.x) → y
        x: direction === "Horizontal" ? node.y : node.x,
        y: direction === "Horizontal" ? node.x : node.y,
      });
    });

    return {
      nodes: nodes.map((node) => {
        const pos = positionMap.get(node.id);
        if (!pos) return node;
        return { ...node, position: pos };
      }),
      edges,
    };
  } catch {
    return { nodes, edges };
  }
};

export const autoLayout = async (
  algorithm: Algorithm = "dagre",
  direction: Direction = "Horizontal",
  nodes: Node[],
  edges: Edge[],
): Promise<{ nodes: Node[]; edges: Edge[] }> => {
  if (algorithm === "elk") return elkLayout(direction, nodes, edges);
  if (algorithm === "d3") return d3Layout(direction, nodes, edges);
  return dagreLayout(direction, nodes, edges);
};
