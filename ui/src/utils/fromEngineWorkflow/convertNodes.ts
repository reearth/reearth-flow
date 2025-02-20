import { config } from "@flow/config";
import { DEFAULT_NODE_SIZE } from "@flow/global-constants";
import { fetcher } from "@flow/lib/fetch/transformers/useFetch";
import type { Action, EngineReadyNode, Node } from "@flow/types";

import { isDefined } from "../isDefined";

export const convertNodes = async (engineNodes?: EngineReadyNode[]) => {
  if (!engineNodes) return [];
  const { api } = config();

  const convertedNodes: Promise<(Node | undefined)[]> = Promise.all(
    engineNodes
      .map(async (en) => {
        let action: Action | undefined;
        if (en.action) {
          action = await fetcher<Action>(`${api}/actions/${en.action}`);
        }

        const canvasNodeType =
          en.type === "subGraph" ? "subworkflow" : action?.type || undefined;

        if (!en.id || !canvasNodeType || !en.name) return undefined;

        const canvasNode: Node = {
          id:
            canvasNodeType === "subworkflow" && en.subGraphId
              ? en.subGraphId
              : en.id,
          type: canvasNodeType,
          position: { x: 0, y: 0 }, // this is temporary before we have a layout
          measured: DEFAULT_NODE_SIZE,
          data: {
            officialName: en.action || en.name,
            customName: en.name,
            params: en.with,
          },
        };
        if (action?.inputPorts.length) {
          canvasNode.data.inputs = action.inputPorts;
        }
        if (action?.outputPorts.length) {
          canvasNode.data.outputs = action.outputPorts;
        }

        return canvasNode;
      })
      .filter(isDefined),
  );
  return convertedNodes;
};
