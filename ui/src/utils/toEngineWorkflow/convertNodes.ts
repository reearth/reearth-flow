import {
  deployableNodeTypes,
  type EngineReadyNode,
  type Node,
} from "@flow/types";

import { isDefined } from "../isDefined";

/**
 * Converts literal escape sequences to actual characters for Rhai engine compatibility on the engine.
 * This fixes the issue where UI sends \\n but engine expects actual newline characters.
 */
const convertEscapeSequences = (obj: any): any => {
  if (typeof obj === "string") {
    return obj
      .replace(/\\n/g, "\n") // Convert \n to actual newline
      .replace(/\\t/g, "\t") // Convert \t to actual tab
      .replace(/\\r/g, "\r") // Convert \r to actual carriage return
      .replace(/\\"/g, '"') // Convert \" to actual quote (though this usually works already)
      .replace(/\\'/g, "'") // Convert \' to actual single quote
      .replace(/\\\\/g, "\\"); // Convert \\ to single backslash
  }

  if (Array.isArray(obj)) {
    return obj.map(convertEscapeSequences);
  }

  if (obj && typeof obj === "object") {
    const result: any = {};
    for (const [key, value] of Object.entries(obj)) {
      result[key] = convertEscapeSequences(value);
    }
    return result;
  }

  return obj;
};

export const convertNodes = (nodes?: Node[]) => {
  if (!nodes) return [];

  const convertedNodes: EngineReadyNode[] = nodes
    .filter(isDeployable)
    .filter(isEnabled)
    ?.map(({ id, type, data }) => {
      if (!id || !type || !data.officialName) return undefined;

      const n: EngineReadyNode = {
        id,
        name: data.officialName,
        type: type === "subworkflow" ? "subGraph" : "action",
      };

      if (data.params) {
        n.with = convertEscapeSequences(data.params);
      }

      if (type === "subworkflow") {
        n.subGraphId = data.subworkflowId;
      } else {
        n.action = data.officialName;
      }

      return n;
    })
    .filter(isDefined);
  console.log("convertedNodes", convertedNodes);
  return convertedNodes;
};

const isDeployable = (node: Node) =>
  node && deployableNodeTypes.includes(node.type);

const isEnabled = (node: Node) => !node.data.isDisabled;
