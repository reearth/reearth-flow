import dagre from "@dagrejs/dagre";

import {
  DEFAULT_BATCH_MIN_SIZE,
  DEFAULT_BATCH_PADDING,
  DEFAULT_LAYOUT_X_SPACING,
  DEFAULT_LAYOUT_Y_SPACING,
  DEFAULT_NODE_SIZE,
} from "@flow/global-constants";
import { Algorithm, Direction, Edge, Node } from "@flow/types";

export type DagreDirection = "TB" | "LR";

type Size = { width: number; height: number };
type Position = { x: number; y: number };
type GraphEdge = { source: string; target: string };

const toPixels = (value: string | number | undefined): number | undefined => {
  const parsed = typeof value === "string" ? parseFloat(value) : value;
  return typeof parsed === "number" && Number.isFinite(parsed) && parsed > 0
    ? parsed
    : undefined;
};

const getNodeSize = (node: Node): Size => ({
  width:
    toPixels(node.style?.width) ??
    toPixels(node.measured?.width) ??
    DEFAULT_NODE_SIZE.width,
  height:
    toPixels(node.style?.height) ??
    toPixels(node.measured?.height) ??
    DEFAULT_NODE_SIZE.height,
});

const layoutWithDagre = (
  nodes: Node[],
  edges: GraphEdge[],
  sizes: Map<string, Size>,
  direction: Direction,
): Map<string, Position> => {
  const isHorizontal = direction === "Horizontal";
  const dagreGraph = new dagre.graphlib.Graph().setDefaultEdgeLabel(() => ({}));

  // In LR layout: ranksep = gap between columns (x), nodesep = gap between rows (y)
  // In TB layout: ranksep = gap between rows (y), nodesep = gap between columns (x)
  dagreGraph.setGraph({
    rankdir: isHorizontal ? "LR" : "TB",
    ranksep: isHorizontal ? DEFAULT_LAYOUT_X_SPACING : DEFAULT_LAYOUT_Y_SPACING,
    nodesep: isHorizontal ? DEFAULT_LAYOUT_Y_SPACING : DEFAULT_LAYOUT_X_SPACING,
  });

  nodes.forEach((node) => {
    dagreGraph.setNode(node.id, sizes.get(node.id) ?? DEFAULT_NODE_SIZE);
  });

  edges.forEach((edge) => {
    dagreGraph.setEdge(edge.source, edge.target);
  });

  dagre.layout(dagreGraph);

  const positions = new Map<string, Position>();
  nodes.forEach((node) => {
    const nodeWithPosition = dagreGraph.node(node.id);
    const size = sizes.get(node.id) ?? DEFAULT_NODE_SIZE;
    // Shift dagre center anchor to React Flow top-left anchor
    positions.set(node.id, {
      x: nodeWithPosition.x - size.width / 2,
      y: nodeWithPosition.y - size.height / 2,
    });
  });

  return positions;
};

export const autoLayout = (
  algorithm: Algorithm = "dagre",
  direction: Direction = "Horizontal",
  nodes: Node[],
  edges: Edge[],
) => {
  if (algorithm !== "dagre") return { nodes, edges };

  const nodeById = new Map(nodes.map((node) => [node.id, node]));

  const parentIdOf = (node: Node | undefined) =>
    node?.parentId && nodeById.has(node.parentId) ? node.parentId : undefined;

  const childrenByParentId = new Map<string, Node[]>();
  nodes.forEach((node) => {
    const parentId = parentIdOf(node);
    if (!parentId) return;
    childrenByParentId.set(parentId, [
      ...(childrenByParentId.get(parentId) ?? []),
      node,
    ]);
  });

  const sizes = new Map(nodes.map((node) => [node.id, getNodeSize(node)]));
  const positions = new Map<string, Position>();

  childrenByParentId.forEach((children, parentId) => {
    const childIds = new Set(children.map((child) => child.id));
    const innerEdges = edges.filter(
      (edge) => childIds.has(edge.source) && childIds.has(edge.target),
    );
    const childPositions = layoutWithDagre(
      children,
      innerEdges,
      sizes,
      direction,
    );

    const minX = Math.min(...[...childPositions.values()].map((p) => p.x));
    const minY = Math.min(...[...childPositions.values()].map((p) => p.y));

    let contentWidth = 0;
    let contentHeight = 0;

    children.forEach((child) => {
      const childPosition = childPositions.get(child.id);
      if (!childPosition) return;
      const size = sizes.get(child.id) ?? DEFAULT_NODE_SIZE;
      const position = {
        x: childPosition.x - minX + DEFAULT_BATCH_PADDING,
        y: childPosition.y - minY + DEFAULT_BATCH_PADDING,
      };
      positions.set(child.id, position);
      contentWidth = Math.max(contentWidth, position.x + size.width);
      contentHeight = Math.max(contentHeight, position.y + size.height);
    });

    sizes.set(parentId, {
      width: Math.max(
        DEFAULT_BATCH_MIN_SIZE.width,
        contentWidth + DEFAULT_BATCH_PADDING,
      ),
      height: Math.max(
        DEFAULT_BATCH_MIN_SIZE.height,
        contentHeight + DEFAULT_BATCH_PADDING,
      ),
    });
  });

  const rootNodes = nodes.filter((node) => !parentIdOf(node));
  const rootIds = new Set(rootNodes.map((node) => node.id));
  const rootIdOf = (nodeId: string) => {
    let currentId: string | undefined = nodeId;
    for (let depth = 0; currentId && depth <= nodes.length; depth++) {
      if (rootIds.has(currentId)) return currentId;
      currentId = parentIdOf(nodeById.get(currentId));
    }
    return undefined;
  };

  const rootEdges = edges.reduce<GraphEdge[]>((acc, edge) => {
    const source = rootIdOf(edge.source);
    const target = rootIdOf(edge.target);
    if (!source || !target || source === target) return acc;
    return [...acc, { source, target }];
  }, []);

  layoutWithDagre(rootNodes, rootEdges, sizes, direction).forEach(
    (position, nodeId) => positions.set(nodeId, position),
  );

  const newNodes: Node[] = nodes.map((node) => {
    const newNode: Node = {
      ...node,
      position: positions.get(node.id) ?? node.position,
    };

    const size = childrenByParentId.has(node.id)
      ? sizes.get(node.id)
      : undefined;
    if (size) {
      newNode.style = {
        ...node.style,
        width: `${size.width}px`,
        height: `${size.height}px`,
      };
    }

    return newNode;
  });

  return { nodes: newNodes, edges };
};
