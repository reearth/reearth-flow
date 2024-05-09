import { Edge as ReactFlowEdge, Node as ReactFlowNode } from "reactflow";

export type Workspace = {
  id: string;
  name: string;
  members: Member[] | undefined;
  projects: Project[] | undefined;
};

export type Member = {
  id: string;
  name: string;
  // status?: "online" | "offline"; // "away" | "idle" ??
  // role: "reader" | "writer" | "admin";
};

export type Project = {
  id: string;
  name: string;
  description?: string;
  workflows: Workflow[] | undefined;
};

export type Workflow = {
  id: string;
  name: string;
  nodes?: Node[];
  edges?: Edge[];
};

export type Status = "success" | "failure" | "active" | "idle"; // other options: pending, warning, loading

export type NodeData = {
  name?: string;
  inputs?: string[];
  outputs?: string[];
  status?: Status;
  // transformer
  transformerId?: string;
  // batch
  content?: string;
};

export type Node = ReactFlowNode<NodeData>;

export type EdgeData = {
  status?: Status;
};

export type Edge = ReactFlowEdge<EdgeData>;
