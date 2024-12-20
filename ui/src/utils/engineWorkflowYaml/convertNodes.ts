import {
  deployableNodeTypes,
  type EngineReadyNode,
  type Node,
} from "@flow/types";

import { isDefined } from "../isDefined";

export const convertNodes = (nodes?: Node[]) => {
  if (!nodes) return [];
  const convertedNodes: EngineReadyNode[] = nodes
    .filter(isDeployable)
    ?.map(({ id, type, data }) => {
      if (!id || !type || !data.officialName) return undefined;

      const n: EngineReadyNode = {
        id,
        name: data.officialName,
        type: type === "subworkflow" ? "subgraph" : "action",
      };

      if (data.params) {
        n.with = data.params;
      }
      if (type === "transformer") {
        n.action = data.officialName; // TODO: Need to assign the action name/id since name will be user customizable
      }
      if (type === "subworkflow") {
        n.subGraphId = id;
      }

      return n;
    })
    .filter(isDefined);
  return convertedNodes;
};

const isDeployable = (node: Node) =>
  node && deployableNodeTypes.includes(node.type);
