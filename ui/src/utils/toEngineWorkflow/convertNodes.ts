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
        type: type === "subworkflow" ? "subGraph" : "action",
      };

      if (data.params) {
        n.with = data.params;
      }

      if (type === "subworkflow") {
        n.subGraphId = data.subworkflowId;
      } else {
        n.action = data.officialName;
      }

      return n;
    })
    .filter(isDefined);
  return convertedNodes;
};

const isDeployable = (node: Node) =>
  node && deployableNodeTypes.includes(node.type);
