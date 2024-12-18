import { XYPosition } from "@xyflow/react";

import { config } from "@flow/config";
import { fetcher } from "@flow/lib/fetch/transformers/useFetch";
import { nodeTypes, type Action, type Node, type NodeType } from "@flow/types";
import { randomID } from "@flow/utils";

import { baseBatchNode } from "./components/Nodes/BatchNode";
import { baseNoteNode } from "./components/Nodes/NoteNode";

type BaseNodeOptions = {
  position: XYPosition;
  type: Action["type"];
};

type CreateNodeFromActionOptions = {
  position: XYPosition;
  action: Action;
};

type CreateNodeOptions = BaseNodeOptions & { action?: Action };

const createBaseNode = ({ position, type }: BaseNodeOptions): Node => ({
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
}: BaseNodeOptions): Node | null => {
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

const createNodeFromAction = ({
  action,
  position,
}: CreateNodeFromActionOptions): Node => ({
  ...createBaseNode({ position, type: action.type }),
  measured: {
    width: 150,
    height: 25,
  },
  data: {
    name: action.name,
    inputs: action.inputPorts ?? [],
    outputs: action.outputPorts ?? [],
    status: "idle",
    locked: false,
  },
});

export const useCreateNode = () => {
  const { api } = config();

  const createActionNode = async (
    name: string,
    position: XYPosition,
  ): Promise<Node | null> => {
    const action = await fetcher<Action>(`${api}/actions/${name}`);
    if (!action) return null;
    return createNodeFromAction({ action, position });
  };

  const createNode = async ({
    position,
    type,
    action,
  }: CreateNodeOptions): Promise<Node | null> => {
    if (action) {
      return createNodeFromAction({ action, position });
    }

    if (nodeTypes.includes(type as NodeType)) {
      return createSpecializedNode({ position, type });
    }

    return createActionNode(type, position);
  };

  return {
    createNode,
    createActionNode,
  };
};

export { createBaseNode, createSpecializedNode, createNodeFromAction };
