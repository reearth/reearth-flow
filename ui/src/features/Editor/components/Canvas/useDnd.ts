import { Dispatch, DragEvent, SetStateAction, useCallback } from "react";
import { Node, useReactFlow } from "reactflow";

import { useRandomId } from "@flow/hooks";

import { baseBatchNode } from "../Nodes/BatchNode";

export default ({ setNodes }: { setNodes: Dispatch<SetStateAction<Node[]>> }) => {
  const reactFlowInstance = useReactFlow();

  const onDragOver = useCallback((event: DragEvent<HTMLDivElement>) => {
    event.preventDefault();
    event.dataTransfer.dropEffect = "move";
  }, []);

  const onDrop = useCallback(
    (event: DragEvent<HTMLDivElement>) => {
      event.preventDefault();

      const type = event.dataTransfer.getData("application/reactflow");

      // check if the dropped element is valid
      if (typeof type === "undefined" || !type) {
        return;
      }

      // reactFlowInstance.project was renamed to reactFlowInstance.screenToFlowPosition
      // and you don't need to subtract the reactFlowBounds.left/top anymore
      // details: https://reactflow.dev/whats-new/2023-11-10
      const position = reactFlowInstance.screenToFlowPosition({
        x: event.clientX,
        y: event.clientY,
      });

      let newNode: Node; // TODO: stronger typing

      newNode = {
        id: useRandomId(),
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
          data: { content: "New Note", width: 300, height: 200 },
        };
      }

      setNodes(nds => nds.concat(newNode));
    },
    [reactFlowInstance, setNodes],
  );

  return { onDragOver, onDrop };
};
