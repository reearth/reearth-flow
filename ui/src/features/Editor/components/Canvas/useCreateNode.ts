import { XYPosition } from "@xyflow/react";

import { config } from "@flow/config";
import { fetcher } from "@flow/lib/fetch/transformers/useFetch";
import { nodeTypes, type Action, type Node, type NodeType } from "@flow/types";
import { randomID } from "@flow/utils";

import { baseBatchNode } from "./components/Nodes/BatchNode";
import { baseNoteNode } from "./components/Nodes/NoteNode";

type CreateNodeOptions = {
  position: XYPosition;
  type: string;
};

const createBaseNode = ({ position, type }: CreateNodeOptions): Node => ({
  id: randomID(),
  position,
  type,
  data: {
    name: type,
    status: "idle",
    locked: false,
  },
});

const createSpecializedNode = ({
  position,
  type,
}: CreateNodeOptions): Node | null => {
  const node = createBaseNode({ position, type });

  switch (type) {
    case "batch":
      return { ...node, ...baseBatchNode };
    case "note":
      return {
        ...node,
        data: { ...node.data, ...baseNoteNode },
      };
    default:
      return node;
  }
};

const createActionNode = async (
  name: string,
  position: XYPosition,
): Promise<Node | null> => {
  const { api } = config();
  const action = await fetcher<Action>(`${api}/actions/${name}`);
  if (!action) return null;

  return {
    ...createBaseNode({ position, type: action.type }),
    // Needs measured, but at time of creation we don't know size yet.
    // 150x25 is base-size of GeneralNode.
    measured: {
      width: 150,
      height: 25,
    },
    data: {
      name: action.name,
      inputs: [...action.inputPorts],
      outputs: [...action.outputPorts],
      status: "idle",
      locked: false,
    },
  };
};

export const useCreateNode = () => {
  const createNode = async ({
    position,
    type,
  }: CreateNodeOptions): Promise<Node | null> => {
    if (nodeTypes.includes(type as NodeType)) {
      return createSpecializedNode({ position, type });
    }
    return createActionNode(type, position);
  };

  return {
    createNode,
  };
};
