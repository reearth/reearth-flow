import type { NodeProps, Node as ReactFlowNode } from "@xyflow/react";
import { ComponentType } from "react";

import type { Status } from "./shared";

type ParamValueType = string | number | boolean | object | null;

// type NodeParam<T extends ParamValueType> = {
type NodeParam = {
  id: string;
  name: string;
  // type: string; perhaps we don't need this
  value: ParamValueType;
};

export type NodeData = {
  name?: string;
  inputs?: string[];
  outputs?: string[];
  status?: Status;
  params?: NodeParam[];
  locked?: boolean | undefined;
  // transformer
  transformerId?: string;
  // batch & note nodes
  content?: string;
  width?: number;
  height?: number;
  backgroundColor?: string;
  textColor?: string;
};

export type NodePosition = { x: number; y: number };

export const actionNodeTypes = ["reader", "writer", "transformer"] as const;

export type ActionNodeType = (typeof actionNodeTypes)[number];

export const deployableNodeTypes = [...actionNodeTypes, "subworkflow"];

export const nodeTypes = [
  ...actionNodeTypes,
  "batch",
  "note",
  "subworkflow",
] as const;

export type NodeType = (typeof nodeTypes)[number];

export type Node = ReactFlowNode<NodeData>;

export type NodeTypes = Record<
  NodeType,
  ComponentType<
    NodeProps & {
      data: NodeData;
      type: NodeType;
    }
  >
>;
