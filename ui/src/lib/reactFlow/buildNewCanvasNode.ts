import { XYPosition } from "@xyflow/react";
import type { JSONSchema7Definition } from "json-schema";

import { config } from "@flow/config";
import { fetcher } from "@flow/lib/fetch/transformers/useFetch";
import { applyDefaults, compile } from "@flow/lib/schemaForm";
import { nodeTypes, type Action, type Node, type NodeType } from "@flow/types";
import { generateUUID } from "@flow/utils";

type CreateNodeOptions = {
  position: XYPosition;
  type: string;
  officialName?: string;
};

type BaseNoteNode = {
  type: NodeType;
  content: string;
  measured: { width: number; height: number };
  style: { width: string; height: string; minWidth: string; minHeight: string };
};

type BaseBatchNode = {
  type: NodeType;
  style: { width: string; height: string };
  zIndex: number;
};

const baseBatchNode: BaseBatchNode = {
  type: "batch",
  style: { width: "300px", height: "200px" },
  zIndex: -1001,
};

const baseNoteNode: BaseNoteNode = {
  type: "note",
  content: "New Note",
  measured: {
    width: 300,
    height: 200,
  },
  style: {
    width: "300px",
    height: "200px",
    minWidth: "250px",
    minHeight: "150px",
  },
};

const createBaseNode = ({
  position,
  type,
  officialName,
}: CreateNodeOptions): Node => ({
  id: generateUUID(),
  position,
  type: type as NodeType,
  data: {
    officialName: officialName || type,
  },
  selected: true,
});

const createSpecializedNode = ({
  position,
  type,
  officialName,
}: CreateNodeOptions): Node | null => {
  const node = createBaseNode({ position, type, officialName });

  switch (type) {
    case "batch":
      return { ...node, ...baseBatchNode };
    case "note":
      return { ...node, ...baseNoteNode };
    default:
      return node;
  }
};

const createActionNode = async (
  name: string,
  position: XYPosition,
): Promise<Node | null> => {
  const { api } = config();
  const action = await fetcher<Action>(
    `${api}/actions/${encodeURIComponent(name)}`,
  );
  if (!action) return null;

  // Seeded from the compiled schema, so a new node starts with the defaults the
  // action states — including those on nested sections, which reading only the
  // top-level `properties` used to miss.
  const defaultParams =
    (applyDefaults(
      compile(action.parameter as JSONSchema7Definition),
      undefined,
    ) as Record<string, any> | undefined) ?? {};

  const newNode = {
    ...createBaseNode({ position, type: action.type }),
    // Needs measured, but at time of creation we don't know size yet.
    // 150x25 is base-size of GeneralNode.
    measured: {
      width: 150,
      height: 25,
    },
    data: {
      officialName: action.name,
      inputs: [...action.inputPorts],
      outputs: [...action.outputPorts],
      params: defaultParams,
      // Baseline schema for migration detection: if the action's schema
      // changes on the API later, ParamEditor compares against this copy.
      paramsSchema: action.parameter,
    },
  };

  return newNode;
};

export const buildNewCanvasNode = async ({
  position,
  type,
  officialName,
}: CreateNodeOptions): Promise<Node | null> => {
  if (nodeTypes.includes(type as NodeType)) {
    return createSpecializedNode({
      position,
      type,
      officialName,
    });
  }
  return createActionNode(type, position);
};
