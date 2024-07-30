import * as Y from "yjs";

import { Edge, Node } from "@flow/types";

export type YWorkflow = Y.Map<Y.Text | YNodesArray | YEdgesArray>;

export type YNodesArray = Y.Array<Node>;

export type YEdgesArray = Y.Array<Edge>;

export const yWorkflowBuilder = (id: string, name: string, nodes?: Node[], edges?: Edge[]) => {
  const yWorkflow = new Y.Map<Y.Text | YNodesArray | YEdgesArray>();

  const yId = new Y.Text(id);
  const yName = new Y.Text(name);
  const yNodes = new Y.Array<Node>();
  if (nodes) {
    yNodes.insert(0, nodes);
  }
  const yEdges = new Y.Array<Edge>();
  if (edges) {
    yEdges.insert(0, edges);
  }

  yWorkflow.set("id", yId);
  yWorkflow.set("name", yName);
  yWorkflow.set("nodes", yNodes);
  yWorkflow.set("edges", yEdges);
  return yWorkflow;
};
