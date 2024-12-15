import type { NodeProps, Node as ReactFlowNode } from "@xyflow/react";
import { ComponentType } from "react";

import type { Status } from "./shared";

// type ParamValueType = string | number | boolean | object | null;

// type NodeParam<T extends ParamValueType> = {
type NodeParam = Record<string, any>;

export type NodeData = {
  officialName: string;
  customName?: string;
  inputs?: string[];
  outputs?: string[];
  status?: Status;
  params?: NodeParam;
  locked?: boolean | undefined;
  // subworkflow nodes
  pseudoInputs?: string[];
  pseudoOutputs?: string[];
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
  "entrance",
  "exit",
] as const;

export type NodeType = (typeof nodeTypes)[number];

export type Node = Omit<ReactFlowNode<NodeData>, "type"> & { type: string };

export type NodeTypes = Record<
  NodeType,
  ComponentType<
    NodeProps & {
      coolName: string;
      data: NodeData;
      type: NodeType;
    }
  >
>;
