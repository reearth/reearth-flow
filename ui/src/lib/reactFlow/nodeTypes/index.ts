import type { NodeTypes } from "@xyflow/react";
import { createElement } from "react";

import BatchNode from "./BatchNode";
import NoteNode from "./NoteNode";
import ReaderNode from "./ReaderNode";
import SubworkflowNode from "./SubworkflowNode";
import TransformerNode from "./TransformerNode";
import WriterNode from "./WriterNode";

export const nodeTypes: NodeTypes = {
  writer: WriterNode,
  reader: ReaderNode,
  transformer: TransformerNode,
  batch: BatchNode,
  note: NoteNode,
  subworkflow: SubworkflowNode,
};

const createFullNodeTypes = (readonly?: boolean): NodeTypes => ({
  writer: (props) => createElement(WriterNode, { ...props, readonly }),
  reader: (props) => createElement(ReaderNode, { ...props, readonly }),
  transformer: (props) =>
    createElement(TransformerNode, { ...props, readonly }),
  subworkflow: (props) =>
    createElement(SubworkflowNode, { ...props, readonly }),
  batch: (props) => createElement(BatchNode, { ...props }),
  note: (props) => createElement(NoteNode, { ...props }),
});

export default createFullNodeTypes;
