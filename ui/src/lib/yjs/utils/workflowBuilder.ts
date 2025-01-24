import * as Y from "yjs";

import type { Edge, Node, NodeData } from "@flow/types";

// Define what is not tracked by Yjs but needed for React Flow
export type NonReactiveFields = {
  // selected?: boolean;
  type: string;
  dragging?: boolean;
  data: NodeData;
};

type NonReactiveField = string | boolean | NodeData;

type YNodeValue = Y.Text | Y.Map<unknown> | number | NonReactiveField; // add other possible types

export type YNode = Y.Map<YNodeValue>;

type YEdgeValue = Y.Text;

export type YEdge = Y.Map<YEdgeValue>;

export type YNodesArray = Y.Array<YNode>;

export type YEdgesArray = Y.Array<YEdge>;

export type YWorkflow = Y.Map<Y.Text | YNodesArray | YEdgesArray>;

// First, define a helper to create a Y.Map for a Node
export const createYNode = (node: Node) => {
  const yNode = new Y.Map() as YNode;

  // Add reactive node properties
  yNode.set("id", new Y.Text(node.id));

  const yPosition = new Y.Map();
  yPosition.set("x", node.position.x);
  yPosition.set("y", node.position.y);
  yNode.set("position", yPosition);

  const yMeasured = new Y.Map();
  yMeasured.set("width", node.measured?.width);
  yMeasured.set("height", node.measured?.height);
  yNode.set("measured", yMeasured);

  if (node.type === "batch") {
    const yStyle = new Y.Map();
    yStyle.set("width", node.style?.width || 0);
    yStyle.set("height", node.style?.height || 0);
    yNode.set("style", yStyle);
  }

  // TODO: figure out how to handle locking

  // All non-reactive properties can be set directly
  const nonReactiveFields: NonReactiveFields = {
    // selected: node.selected,
    type: node.type,
    dragging: node.dragging,
    data: { ...node.data },
  };
  yNode.set("type", nonReactiveFields["type"]);
  // yNode.set("selected", nonReactiveFields.selected || false);
  yNode.set("dragging", nonReactiveFields["dragging"] || false);
  yNode.set("data", nonReactiveFields["data"]);

  return yNode;
};

export const createYEdge = (edge: Edge) => {
  const yEdge = new Y.Map() as YEdge;

  yEdge.set("id", new Y.Text(edge.id));
  yEdge.set("source", new Y.Text(edge.source));
  yEdge.set("target", new Y.Text(edge.target));
  if (edge.sourceHandle) {
    yEdge.set("sourceHandle", new Y.Text(edge.sourceHandle));
  }
  if (edge.targetHandle) {
    yEdge.set("targetHandle", new Y.Text(edge.targetHandle));
  }

  return yEdge;
};

export const yWorkflowBuilder = (
  id: string,
  name: string,
  nodes?: Node[],
  edges?: Edge[],
) => {
  const yWorkflow = new Y.Map() as YWorkflow;
  const yId = new Y.Text(id);
  const yName = new Y.Text(name);
  const yNodes = new Y.Array() as YNodesArray;
  const yEdges = new Y.Array() as YEdgesArray;

  if (nodes) {
    const yNodeMaps = nodes.map((node) => createYNode(node));
    yNodes.insert(0, yNodeMaps);
  }

  if (edges) {
    const yEdgeMaps = edges?.map((edge) => createYEdge(edge));
    yEdges.insert(0, yEdgeMaps);
  }

  yWorkflow.set("id", yId);
  yWorkflow.set("name", yName);
  yWorkflow.set("nodes", yNodes);
  yWorkflow.set("edges", yEdges);
  return yWorkflow;
};
