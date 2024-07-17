import { useReactFlow } from "@xyflow/react";
import { Dispatch, DragEvent, SetStateAction, useCallback } from "react";

import { Node } from "@flow/types";
import { randomID } from "@flow/utils";

import { baseBatchNode } from "./components/Nodes/BatchNode";
import { baseNoteNode } from "./components/Nodes/NoteNode";

type Props = {
  setNodes: Dispatch<SetStateAction<Node[]>>;
  onNodeLocking: (nodeId: string, setNodes: Dispatch<SetStateAction<Node[]>>) => void;
};

// This is used for drag and drop functionality in to the canvas
// This is not used for node dnd within the canvas. That is done internally by react-flow
export default ({ setNodes, onNodeLocking }: Props) => {
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

      const handleNodeLocking = (setNodes: Dispatch<SetStateAction<Node[]>>) => (nodeId: string) =>
        onNodeLocking(nodeId, setNodes);

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
          onLock: handleNodeLocking(setNodes),
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

      setNodes(nds => nds.concat(newNode));
    },
    [onNodeLocking, screenToFlowPosition, setNodes],
  );

  return { handleNodeDragOver, handleNodeDrop };
};
