import { NodeTypes } from "reactflow";

import ReaderNode from "./ReaderNode";
import TransformerNode from "./TransformerNode";
import WriterNode from "./WriterNode";

const nodeTypes: NodeTypes = {
  writer: WriterNode,
  reader: ReaderNode,
  transformer: TransformerNode,
};

export default nodeTypes;
