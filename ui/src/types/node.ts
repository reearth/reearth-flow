import type { NodeProps, Node as ReactFlowNode } from "@xyflow/react";
import { ComponentType } from "react";

import type { Status } from "./shared";

type ParamValueType = string | number | boolean | object | null;

type NodeParam<T extends ParamValueType> = {
  id: string;
  name: string;
  // type: string; perhaps we don't need this
  value: T;
};

export type NodeData = {
  name?: string;
  inputs?: string[];
  outputs?: string[];
  status?: Status;
  params?: NodeParam<any>[];
  locked?: boolean | undefined;
  onDoubleClick?: (nodeId: string) => void;
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

export type NodeType = "reader" | "writer" | "transformer" | "batch" | "note" | "subworkflow";

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
