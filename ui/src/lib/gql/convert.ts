import { DEFAULT_EDGE_PORT } from "@flow/global-constants";
import type { Workflow } from "@flow/types";

import {
  InputWorkflow,
  InputWorkflowEdge,
  InputWorkflowNode,
} from "./__gen__/graphql";

export const toGQLWorkflow = ({
  projectId,
  name,
  workflows,
}: {
  projectId: string;
  name?: string;
  workflows: Workflow[];
}): InputWorkflow => {
  const gqlWorkflow: InputWorkflow = {
    id: projectId,
    name: name ?? "untitled workflow",
    graphs: [],
    entryGraphId: workflows[0]?.id,
  };
  for (const w of workflows) {
    const nodes: InputWorkflowNode[] =
      w.nodes?.map((node): InputWorkflowNode => {
        const newNode: InputWorkflowNode = {
          id: node.id,
          type: node.type,
          name: node.data.name ?? "undefinedAction",
        };
        if (node.type === "subworkflow") {
          // newNode.subGraphId =
        }
        return newNode;
      }) ?? [];
    const edges: InputWorkflowEdge[] =
      w.edges?.map((edge) => ({
        id: edge.id,
        from: edge.source,
        to: edge.target,
        fromPort: edge.sourceHandle ?? DEFAULT_EDGE_PORT,
        toPort: edge.targetHandle ?? DEFAULT_EDGE_PORT,
      })) ?? [];
    gqlWorkflow.graphs.push({
      id: w.id,
      name: w.name ?? "undefined",
      nodes: nodes,
      edges: edges,
    });
  }
  return gqlWorkflow;
};
