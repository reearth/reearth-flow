import * as Y from "yjs";

import { Edge, Node } from "@flow/types";

export type YWorkflow = Y.Map<Y.Text | YNodesArray | YEdgesArray>;

export type YNodesArray = Y.Array<Node>;

export type YEdgesArray = Y.Array<Edge>;

export const yWorkflowBuilder = (id: string, name: string) => {
  const yWorkflow = new Y.Map<Y.Text | YNodesArray | YEdgesArray>();

  const yId = new Y.Text(id);
  const yName = new Y.Text(name);
  const yNodes = new Y.Array<Node>();
  const yEdges = new Y.Array<Edge>();

  yWorkflow.set("id", yId);
  yWorkflow.set("name", yName);
  yWorkflow.set("nodes", yNodes);
  yWorkflow.set("edges", yEdges);
  return yWorkflow;
};
