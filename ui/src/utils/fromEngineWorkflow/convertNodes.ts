import { config } from "@flow/config";
import { DEFAULT_NODE_SIZE } from "@flow/global-constants";
import { fetcher } from "@flow/lib/fetch/transformers/useFetch";
import type {
  Action,
  EngineReadyNode,
  Node,
  NodeType,
  PseudoPort,
} from "@flow/types";

export const convertNodes = async (
  engineNodes: EngineReadyNode[],
  getSubworkflowPseudoPorts: (id: string) =>
    | {
        pseudoInputs: PseudoPort[];
        pseudoOutputs: PseudoPort[];
      }
    | undefined,
) => {
  if (!engineNodes) return [];
  const { api } = config();

  const convertedNodes: Promise<(Node | undefined)[]> = Promise.all(
    engineNodes.map(async (en) => {
      let action: Action | undefined;
      if (en.action) {
        action = await fetcher<Action>(`${api}/actions/${en.action}`);
      }

      const canvasNodeType =
        en.type === "subGraph" ? "subworkflow" : action?.type || undefined;

      const isSubworkflow = en.type === "subGraph";

      if (!en.id || !canvasNodeType || !en.name) return undefined;

      const canvasNode: Node = {
        id: en.id,
        type: (canvasNodeType as NodeType) || undefined,
        position: { x: 0, y: 0 }, // this is temporary before we have a layout
        measured: DEFAULT_NODE_SIZE,
        data: {
          officialName: isSubworkflow ? "Subworkflow" : en.action || en.name,
          params: en.with,
          customizations: {
            customName: en.name,
          },
        },
      };

      if (isSubworkflow && en.subGraphId) {
        canvasNode.data.subworkflowId = en.subGraphId;

        const subworkflowPseudoPorts = getSubworkflowPseudoPorts(en.subGraphId);
        if (subworkflowPseudoPorts) {
          canvasNode.data.pseudoInputs = subworkflowPseudoPorts.pseudoInputs;
          canvasNode.data.pseudoOutputs = subworkflowPseudoPorts.pseudoOutputs;
        }
      }

      if (action?.inputPorts.length) {
        canvasNode.data.inputs = action.inputPorts;
      }
      if (action?.outputPorts.length) {
        canvasNode.data.outputs = action.outputPorts;
      }

      return canvasNode;
    }),
  );
  return convertedNodes;
};
