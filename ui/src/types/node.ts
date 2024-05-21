import type { Node as ReactFlowNode } from "reactflow";

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
  // transformer
  transformerId?: string;
  // batch
  content?: string;
};

export type Node = ReactFlowNode<NodeData>;
