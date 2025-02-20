import { DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";
import type {
  Workflow,
  EngineReadyWorkflow,
  EngineReadyGraph,
} from "@flow/types";

import { autoLayout } from "../autoLayout";
import { isDefined } from "../isDefined";

import { convertEdges } from "./convertEdges";
import { convertNodes } from "./convertNodes";

export const separateWorkflow = async (
  engineWorkflow: EngineReadyWorkflow,
): Promise<Workflow[] | undefined> => {
  const { graphs, entryGraphId } = engineWorkflow;
  if (graphs.some((graph) => !isWorkflowGraph(graph))) {
    throw new Error("Invalid graph found in engine workflow");
  }

  const workflowsPromises = graphs.map(async (graph: EngineReadyGraph) => {
    const nodes = (await convertNodes(graph.nodes)).filter(isDefined);
    const edges = convertEdges(graph.edges);

    const { nodes: layoutedNodes, edges: layoutedEdges } = autoLayout(
      "dagre",
      "Horizontal",
      nodes,
      edges,
    );

    return {
      id: graph.id === entryGraphId ? DEFAULT_ENTRY_GRAPH_ID : graph.id,
      name: graph.name,
      nodes: layoutedNodes,
      edges: layoutedEdges,
    };
  });

  const workflows = await Promise.all(workflowsPromises);

  return workflows;
};

// Helper type guard to check if a graph has required workflow properties
const isWorkflowGraph = (graph: EngineReadyGraph): boolean => {
  return (
    typeof graph === "object" &&
    graph !== null &&
    "nodes" in graph &&
    "edges" in graph
  );
};
