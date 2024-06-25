import { useReactFlow } from "@xyflow/react";
import { Dispatch, DragEvent, SetStateAction } from "react";

import { Node } from "@flow/types";
import { randomID } from "@flow/utils";

import { baseBatchNode } from "../Nodes/BatchNode";
import { baseNoteNode } from "../Nodes/NoteNode";

export default ({ setNodes }: { setNodes: Dispatch<SetStateAction<Node[]>> }) => {
  const { screenToFlowPosition } = useReactFlow();

  const onDragOver = (event: DragEvent<HTMLDivElement>) => {
    event.preventDefault();
    event.dataTransfer.dropEffect = "move";
  };

  const onDrop = (event: DragEvent<HTMLDivElement>) => {
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

  return { onDragOver, onDrop };
};
