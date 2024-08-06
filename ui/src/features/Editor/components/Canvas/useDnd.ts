import { useReactFlow } from "@xyflow/react";
import { DragEvent, useCallback } from "react";

import { config } from "@flow/config";
import { fetcher } from "@flow/lib/fetch/transformers/useFetch";
import { Action, Node } from "@flow/types";
import { randomID } from "@flow/utils";

import { baseBatchNode } from "./components/Nodes/BatchNode";
import { baseNoteNode } from "./components/Nodes/NoteNode";

type Props = {
  nodes: Node[];
  onNodesChange: (nodes: Node[]) => void;
  onNodeLocking: (nodeId: string) => void;
};

// This is used for drag and drop functionality in to the canvas
// This is not used for node dnd within the canvas. That is done internally by react-flow
export default ({ nodes, onNodesChange, onNodeLocking }: Props) => {
  const { screenToFlowPosition } = useReactFlow();
  const { api } = config();

  const handleNodeDragOver = useCallback((event: DragEvent<HTMLDivElement>) => {
    event.preventDefault();
    event.dataTransfer.dropEffect = "move";
  }, []);

  const handleNodeDrop = useCallback(
    async (event: DragEvent<HTMLDivElement>) => {
      event.preventDefault();

      const actionName = event.dataTransfer.getData("application/reactflow");

      // check if the dropped element is valid
      if (typeof actionName === "undefined" || !actionName) return;

      const position = screenToFlowPosition({
        x: event.clientX,
        y: event.clientY,
      });

      const action = await fetcher<Action>(`${api}/actions/${actionName}`);
      if (!action) return;

      let newNode: Node = {
        id: randomID(),
        type: action.type,
        position,
        data: {
          name: action.name,
          inputs: [...action.inputPorts],
          outputs: [...action.outputPorts],
          status: "idle",
          locked: false,
          onDoubleClick: onNodeLocking,
        },
      };

      if (action.type === "batch") {
        newNode = { ...newNode, ...baseBatchNode };
      }

      if (action.type === "note") {
        newNode = {
          ...newNode,
          data: { ...newNode.data, ...baseNoteNode },
        };
      }

      onNodesChange(nodes.concat(newNode));
    },
    [nodes, api, onNodeLocking, screenToFlowPosition, onNodesChange]
  );

  return { handleNodeDragOver, handleNodeDrop };
};
