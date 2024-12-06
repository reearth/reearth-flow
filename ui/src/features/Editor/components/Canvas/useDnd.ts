import { useReactFlow, XYPosition } from "@xyflow/react";
import { DragEvent, useCallback } from "react";

import { config } from "@flow/config";
import { fetcher } from "@flow/lib/fetch/transformers/useFetch";
import {
  nodeTypes,
  type ActionNodeType,
  type Action,
  type Node,
  type NodeType,
} from "@flow/types";
import { randomID } from "@flow/utils";

import { baseBatchNode } from "./components/Nodes/BatchNode";
import { baseNoteNode } from "./components/Nodes/NoteNode";

type Props = {
  nodes: Node[];
  onWorkflowAdd: (position?: XYPosition) => void;
  onNodesChange: (nodes: Node[]) => void;
  onNodePickerOpen: (position: XYPosition, nodeType?: ActionNodeType) => void;
};

// This is used for drag and drop functionality in to the canvas
// This is not used for node dnd within the canvas. That is done internally by react-flow
export default ({
  nodes,
  onWorkflowAdd,
  onNodesChange,
  onNodePickerOpen,
}: Props) => {
  const { screenToFlowPosition } = useReactFlow();
  const { api } = config();

  const handleNodeDragOver = useCallback((event: DragEvent<HTMLDivElement>) => {
    event.preventDefault();
    event.dataTransfer.dropEffect = "move";
  }, []);

  const handleNodeDrop = useCallback(
    async (event: DragEvent<HTMLDivElement>) => {
      event.preventDefault();

      const position = screenToFlowPosition({
        x: event.clientX,
        y: event.clientY,
      });

      const d = event.dataTransfer.getData("application/reactflow");

      // check if the dropped element is valid
      if (typeof d === "undefined" || !d) return;

      if (d === "subworkflow") {
        onWorkflowAdd(position);
        return;
      }

      let newNode: Node = {
        id: randomID(),
        position,
        type: d,
        data: {
          name: d,
          status: "idle",
          locked: false,
        },
      };

      if (nodeTypes.includes(d as NodeType)) {
        newNode = {
          ...newNode,
          type: d,
          data: {
            ...newNode.data,
          },
        };

        if (d === "batch") {
          newNode = { ...newNode, ...baseBatchNode };
        } else if (d === "note") {
          newNode = {
            ...newNode,
            data: { ...newNode.data, ...baseNoteNode },
          };
        } else {
          onNodePickerOpen(position, d as ActionNodeType);
          return;
        }
      } else {
        const action = await fetcher<Action>(`${api}/actions/${d}`);
        if (!action) return;

        newNode = {
          ...newNode,
          type: action.type,
          data: {
            ...newNode.data,
            name: action.name,
            inputs: [...action.inputPorts],
            outputs: [...action.outputPorts],
          },
        };
      }

      console.log("newNode", newNode);

      onNodesChange(nodes.concat(newNode));
    },
    [
      nodes,
      api,
      onWorkflowAdd,
      screenToFlowPosition,
      onNodesChange,
      onNodePickerOpen,
    ],
  );

  return { handleNodeDragOver, handleNodeDrop };
};
