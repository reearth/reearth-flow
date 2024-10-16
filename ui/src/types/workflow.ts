import type { Edge } from "./edge";
import type { Node } from "./node";

export type Workflow = {
  id: string;
  name?: string;
  nodes?: Node[];
  edges?: Edge[];
  createdAt?: string;
  updatedAt?: string;
  // projectId?: string;
  // workspaceId?: string;
  // status??
  // params?: any;
};

export type EngineReadyWorkflow = {
  id: string;
  name: string;
  entryGraphId: string;
  with?: any; // TODO: better type this (if possible)
  graphs: EngineReadyGraph[];
};

export type EngineReadyGraph = {
  id: string;
  name: string;
  nodes: EngineReadyNode[];
  edges: EngineReadyEdge[];
};

export type EngineReadyNode = {
  id: string;
  name: string;
  type: string;
  subGraphId?: string;
  action?: string;
  with?: any; // TODO: better type this (if possible)
};

export type EngineReadyEdge = {
  id: string;
  from: string;
  to: string;
  fromPort: string;
  toPort: string;
};
