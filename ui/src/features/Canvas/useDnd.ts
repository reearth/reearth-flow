import { Dispatch, DragEvent, SetStateAction, useCallback, useState } from "react";
import { Node } from "reactflow";

import { baseBatchNode } from "./components/Nodes/BatchNode";

export default ({ setNodes }: { setNodes: Dispatch<SetStateAction<Node[]>> }) => {
  const [reactFlowInstance, setReactFlowInstance] = useState<any>(null);

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

      let newNode: Node;

      newNode = {
        id: createRandomId(),
        type,
        position,
        data: { name: `New ${type} node`, inputs: ["source"], outputs: ["target"] },
      };

      if (type === "batch") {
        newNode = { ...newNode, ...baseBatchNode };
      }

      if (type === "note") {
        newNode = { ...newNode, data: { content: "New Note", width: 300, height: 200 } };
      }

      setNodes(nds => nds.concat(newNode));
    },
    [reactFlowInstance, setNodes],
  );

  return { onDragOver, onDrop, setReactFlowInstance };
};

function createRandomId(length = 10): string {
  let result = "";
  const characters = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
  const charactersLength = characters.length;

  for (let i = 0; i < length; i++) {
    result += characters.charAt(Math.floor(Math.random() * charactersLength));
  }

  return result;
}
