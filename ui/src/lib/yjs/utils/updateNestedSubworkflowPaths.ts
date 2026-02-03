import * as Y from "yjs";

import { Node } from "@flow/types";

import { YNodesMap, YWorkflow } from "../types";

export function updateNestedSubworkflowPaths(
  yWorkflows: Y.Map<YWorkflow>,
  nodeList: Node[],
  basePath: string,
) {
  nodeList.forEach((node) => {
    if (node.type === "subworkflow" && node.data.subworkflowId) {
      const nestedWorkflow = yWorkflows.get(node.data.subworkflowId);
      if (nestedWorkflow) {
        const nestedPath = basePath
          ? `${basePath}.${node.data.subworkflowId}`
          : node.data.subworkflowId;

        const nestedYNodes = nestedWorkflow.get("nodes") as YNodesMap;
        if (nestedYNodes) {
          const nestedNodes: Node[] = [];
          nestedYNodes.forEach((yNode, nodeId) => {
            const yData = yNode.get("data") as Y.Map<any>;
            if (yData) {
              const newPathText = new Y.Text();
              newPathText.insert(0, nestedPath);
              yData.set("workflowPath", newPathText);

              const nodeType = yNode.get("type")?.toString();
              const subworkflowId = yData.get("subworkflowId")?.toString();
              if (nodeType === "subworkflow" && subworkflowId) {
                nestedNodes.push({
                  id: nodeId,
                  type: "subworkflow",
                  position: { x: 0, y: 0 },
                  data: {
                    officialName: "",
                    subworkflowId,
                  },
                });
              }
            }
          });

          if (nestedNodes.length > 0) {
            updateNestedSubworkflowPaths(yWorkflows, nestedNodes, nestedPath);
          }
        }
      }
    }
  });
}
