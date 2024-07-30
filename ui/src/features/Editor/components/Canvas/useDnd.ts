import { useReactFlow } from "@xyflow/react";
import { DragEvent, useCallback } from "react";

import { Node } from "@flow/types";
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

  const handleNodeDragOver = useCallback((event: DragEvent<HTMLDivElement>) => {
    event.preventDefault();
    event.dataTransfer.dropEffect = "move";
  }, []);

  const handleNodeDrop = useCallback(
    (event: DragEvent<HTMLDivElement>) => {
      event.preventDefault();

      const type = event.dataTransfer.getData("application/reactflow");

      // check if the dropped element is valid
      if (typeof type === "undefined" || !type) return;

      const position = screenToFlowPosition({
        x: event.clientX,
        y: event.clientY,
      });

      let newNode: Node;

      // TODO: Once we have access to transformers list, we can set the newNode based on the selected type
      newNode = {
        id: randomID(),
        type,
        position,
        data: {
          name: `New ${type} node`,
          inputs: type === "reader" ? undefined : ["source"],
          outputs: type === "writer" ? undefined : ["target"],
          status: "idle",
          locked: false,
          onDoubleClick: onNodeLocking,
        },
      };

      if (type === "batch") {
        newNode = { ...newNode, ...baseBatchNode };
      }

      if (type === "note") {
        newNode = {
          ...newNode,
          data: { ...newNode.data, ...baseNoteNode },
        };
      }

      onNodesChange(nodes.concat(newNode));
    },
    [nodes, onNodeLocking, screenToFlowPosition, onNodesChange],
  );

  return { handleNodeDragOver, handleNodeDrop };
};
