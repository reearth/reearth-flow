import { useReactFlow } from "@xyflow/react";
import { Dispatch, DragEvent, SetStateAction } from "react";

import { Node } from "@flow/types";
import { randomID } from "@flow/utils";

import { baseBatchNode } from "../Nodes/BatchNode";
import { baseNoteNode } from "../Nodes/NoteNode";

// This is used for drag and drop functionality in to the canvas
// This is not used for node dnd within the canvas. That is done internally by react-flow
export default ({ setNodes }: { setNodes: Dispatch<SetStateAction<Node[]>> }) => {
  const { screenToFlowPosition } = useReactFlow();

  const handleNodeDragOver = (event: DragEvent<HTMLDivElement>) => {
    event.preventDefault();
    event.dataTransfer.dropEffect = "move";
  };

  const handleNodeDrop = (event: DragEvent<HTMLDivElement>) => {
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
      data: { name: `New ${type} node`, inputs: ["source"], outputs: ["target"], status: "idle" },
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

    setNodes(nds => nds.concat(newNode));
  };

  return { handleNodeDragOver, handleNodeDrop };
};
