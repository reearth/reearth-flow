import * as Y from "yjs";

import { Workflow } from "@flow/types";

import { reassembleEdge, reassembleNode } from "./convertToRawWorkflow";
import { YEdge, YNode, YWorkflow } from "./workflowBuilder";

export const convertYWorkflowToWorkflow = (yWorkflow: YWorkflow): Workflow => {
  const workflow: Workflow = {
    id: "", // Default value, update if `id` is found in `yWorkflow`
  };

  // Iterate over the YWorkflow entries
  yWorkflow.forEach((value, key) => {
    if (key === "id" && value instanceof Y.Text) {
      workflow.id = value.toString();
    } else if (key === "name" && value instanceof Y.Text) {
      workflow.name = value.toString();
    } else if (key === "nodes" && value instanceof Y.Array) {
      // Convert nodes to plain objects
      workflow.nodes = value
        .toArray()
        .map((yNode) => reassembleNode(yNode as YNode));
    } else if (key === "edges" && value instanceof Y.Array) {
      // Convert edges to plain objects
      workflow.edges = value
        .toArray()
        .map((yEdge) => reassembleEdge(yEdge as YEdge));
    } else if (key === "createdAt" && value instanceof Y.Text) {
      workflow.createdAt = value.toString();
    } else if (key === "updatedAt" && value instanceof Y.Text) {
      workflow.updatedAt = value.toString();
    }
  });

  return workflow;
};
