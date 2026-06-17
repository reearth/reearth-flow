import type { RJSFSchema } from "@rjsf/utils";
import type {
  Node as ReactFlowNode,
  NodeChange as ReactFlowNodeChange,
} from "@xyflow/react";

import type { NodeSchemaMeta } from "./schemaPreview";

/** Arbitrary per-node metadata persisted (collaboratively) on the Yjs node. */
export type NodeMetadata = {
  /** Output attribute schema as probed by the engine (readers). */
  schema?: NodeSchemaMeta;
};

export type NodeParams = Record<string, any>;

// TODO: Add generic for NodeCustomization for better type checking and separation of concerns
export type NodeCustomizations = {
  customName?: string;
  content?: string;
  backgroundColor?: string;
  titleColor?: string;
};

export type PseudoPort = {
  nodeId: string;
  portName: string;
};

export type NodeData = {
  officialName: string;
  inputs?: string[];
  outputs?: string[];
  params?: NodeParams;
  paramsSchema?: RJSFSchema;
  customizations?: NodeCustomizations;
  metadata?: NodeMetadata;
  workflowPath?: string;
  isCollapsed?: boolean;
  isDisabled?: boolean;
  // subworkflow nodes
  subworkflowId?: string;
  pseudoInputs?: PseudoPort[];
  pseudoOutputs?: PseudoPort[];
};

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

export type NodeChange = ReactFlowNodeChange<Node>;
