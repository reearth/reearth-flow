import { NodeTypes } from "reactflow";

import BatchNode from "./BatchNode";
import ReaderNode from "./ReaderNode";
import TransformerNode from "./TransformerNode";
import WriterNode from "./WriterNode";

export const nodeTypes: NodeTypes = {
  writer: WriterNode,
  reader: ReaderNode,
  transformer: TransformerNode,
  batch: BatchNode,
};
