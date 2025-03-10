import type {
  NodeProps,
  Node as ReactFlowNode,
  NodeChange as ReactFlowNodeChange,
} from "@xyflow/react";
import { ComponentType } from "react";

type NodeParam = Record<string, any>;

export type PseudoPort = {
  nodeId: string;
  portName: string;
};

export type NodeData = {
  officialName: string;
  customName?: string;
  inputs?: string[];
  outputs?: string[];
  params?: NodeParam;
  // subworkflow nodes
  subworkflowId?: string;
  pseudoInputs?: PseudoPort[];
  pseudoOutputs?: PseudoPort[];
  // batch & note nodes
  content?: string;
  backgroundColor?: string;
  textColor?: string;
};

export type NodePosition = { x: number; y: number };

export const actionNodeTypes = ["reader", "writer", "transformer"] as const;

export type ActionNodeType = (typeof actionNodeTypes)[number];

export const isActionNodeType = (value: string): value is ActionNodeType => {
  return actionNodeTypes.includes(value as ActionNodeType);
};

export const deployableNodeTypes = [...actionNodeTypes, "subworkflow"];

export const nodeTypes = [
  ...actionNodeTypes,
  "batch",
  "note",
  "subworkflow",
] as const;

export type NodeType = (typeof nodeTypes)[number];

export type Node = Omit<ReactFlowNode<NodeData>, "type"> & { type: NodeType };

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

export type NodeChange = ReactFlowNodeChange<Node>;
