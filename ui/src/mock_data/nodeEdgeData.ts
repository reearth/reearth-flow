import { Edge, Node } from "reactflow";

import { baseBatchNode } from "@flow/features/Canvas/components/Nodes/BatchNode";
import { initialSize } from "@flow/features/Canvas/components/Nodes/NoteNode";

export const initialNodes: Node[] = [
  {
    id: "1",
    type: "reader",
    data: { name: "Reader Node 1 asdf asdf", outputs: ["output1", "output2"] },
    position: { x: 10, y: 1 },
  },
  {
    id: "1-transformer",
    type: "transformer",
    data: {
      name: "Tranformer Node 1-1",
      inputs: ["input1", "input2", "input3"],
      outputs: ["output1", "output2", "output3"],
    },
    position: { x: 300, y: 1 },
  },
  {
    id: "2-transformer",
    type: "transformer",
    data: { name: "Tranformer Node 1-2", inputs: ["input1"], outputs: ["output1"] },
    position: { x: 600, y: 100 },
  },
  {
    id: "2",
    type: "transformer",
    selected: true,
    data: { name: "Transformer Node 2", inputs: ["input1"], outputs: ["output1", "output2"] },
    // position: { x: 315, y: 400 }, // Original position
    position: { x: 115, y: 100 }, // OP - parent's position
    parentNode: "batch1",
  },
  {
    id: "3",
    type: "writer",
    data: { name: "Writer Node 3", inputs: ["input1", "input2"] },
    position: { x: 705, y: 400 },
  },
  {
    id: "4",
    type: "writer",
    data: { name: "Writer Node 4", inputs: ["input1"] },
    position: { x: 605, y: 500 },
  },
  {
    id: "5",
    type: "writer",
    data: { name: "Writer Node 5", inputs: ["input1"] },
    position: { x: 900, y: 50 },
  },
  {
    id: "6",
    type: "reader",
    data: { name: "Reader Node 6", outputs: ["output1"] },
    // position: { x: 210, y: 350 }, // Original position
    position: { x: 10, y: 50 }, // OP - parent's position
    parentNode: "batch1",
    extent: "parent",
  },
  {
    ...baseBatchNode,
    id: "batch1",
    data: { name: "Batch 1" },
    position: { x: 200, y: 300 },
  },
  {
    id: "note1",
    type: "note",
    data: { content: "Note 1" },
    position: { x: 900, y: 100 },
    style: { width: initialSize.width + "px", height: initialSize.height + "px" },
    zIndex: -1001,
  },
];

export const initialEdges: Edge[] = [
  { id: "e1-1t", source: "1", target: "1-transformer" },
  { id: "e1t-2t", source: "1-transformer", target: "2-transformer", sourceHandle: "output3" },
  { id: "e2t-5", source: "2-transformer", target: "5" },
  { id: "e2-3", source: "2", target: "3", targetHandle: "input2" },
  { id: "e2-4", source: "2", target: "4", sourceHandle: "output2", targetHandle: "input1" },
  { id: "e6-2", source: "6", target: "2" },
  // { id: "e6-t2", source: "6", target: "2", sourceHandle: "input1" },
];

// ReactFlow Node typings
// export type Node<T = any, U extends string | undefined = string | undefined> = {
//   id: string;
//   position: XYPosition;
//   data: T;
//   type?: U;
//   style?: CSSProperties;
//   className?: string;
//   sourcePosition?: Position;
//   targetPosition?: Position;
//   hidden?: boolean;
//   selected?: boolean;
//   dragging?: boolean;
//   draggable?: boolean;
//   selectable?: boolean;
//   connectable?: boolean;
//   deletable?: boolean;
//   dragHandle?: string;
//   width?: number | null;
//   height?: number | null;
//   parentNode?: string;
//   zIndex?: number;
//   extent?: 'parent' | CoordinateExtent;
//   expandParent?: boolean;
//   positionAbsolute?: XYPosition;
//   ariaLabel?: string;
//   focusable?: boolean;
//   resizing?: boolean;
//   [internalsSymbol]?: {
//       z?: number;
//       handleBounds?: NodeHandleBounds;
//       isParent?: boolean;
//   };
// };

// ReactFlow Edge typings
// type DefaultEdge<T = any> = {
//   id: string;
//   type?: string;
//   source: string;
//   target: string;
//   sourceHandle?: string | null;
//   targetHandle?: string | null;
//   style?: CSSProperties;
//   animated?: boolean;
//   hidden?: boolean;
//   deletable?: boolean;
//   data?: T;
//   className?: string;
//   sourceNode?: Node;
//   targetNode?: Node;
//   selected?: boolean;
//   markerStart?: EdgeMarkerType;
//   markerEnd?: EdgeMarkerType;
//   zIndex?: number;
//   ariaLabel?: string;
//   interactionWidth?: number;
//   focusable?: boolean;
//   updatable?: EdgeUpdatable;
// } & EdgeLabelOptions;

// type EdgeLabelOptions = {
//   label?: string | ReactNode;
//   labelStyle?: CSSProperties;
//   labelShowBg?: boolean;
//   labelBgStyle?: CSSProperties;
//   labelBgPadding?: [number, number];
//   labelBgBorderRadius?: number;
// };
