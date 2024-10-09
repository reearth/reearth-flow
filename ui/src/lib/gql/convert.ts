import type { Workflow } from "@flow/types";

import {
  InputWorkflow,
  InputWorkflowEdge,
  InputWorkflowNode,
} from "./__gen__/graphql";

const DEFAULT_PORT = "default";

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
          name: node.data.name ?? "undefined node",
          type: node.type,
        };
        if (node.type === "subworkflow") {
          // newNode.subGraphId =
        }
        return {
          id: node.id,
          name: node.data.name ?? "undefined node",
          type: node.type,
          action: node.data.name, // this might be wrong

          // parameters: node.data.params,
        };
      }) ?? [];
    const edges: InputWorkflowEdge[] =
      w.edges?.map((edge) => ({
        id: edge.id,
        from: edge.source,
        to: edge.target,
        fromPort: edge.sourceHandle ?? DEFAULT_PORT,
        toPort: edge.targetHandle ?? DEFAULT_PORT,
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
