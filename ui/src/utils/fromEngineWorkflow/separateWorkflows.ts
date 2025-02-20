import {
  DEFAULT_ENTRY_GRAPH_ID,
  DEFAULT_ROUTING_PORT,
} from "@flow/global-constants";
import type {
  Workflow,
  EngineReadyWorkflow,
  EngineReadyGraph,
  Algorithm,
  PseudoPort,
} from "@flow/types";

import { autoLayout } from "../autoLayout";
import { isDefined } from "../isDefined";

import { convertEdges } from "./convertEdges";
import { convertNodes } from "./convertNodes";

export const separateWorkflow = async ({
  engineWorkflow,
  layoutType,
}: {
  engineWorkflow: EngineReadyWorkflow;
  layoutType?: Algorithm;
}): Promise<Workflow[] | undefined> => {
  const { graphs, entryGraphId } = engineWorkflow;
  if (graphs.some((graph) => !isWorkflowGraph(graph))) {
    throw new Error("Invalid graph found in engine workflow");
  }

  const getSubworkflowPseudoPorts = (id: string) => {
    const workflow = graphs.find((graph) => graph.id === id);
    if (!workflow) return;

    const pseudoInputs: PseudoPort[] = [];
    const pseudoOutputs: PseudoPort[] = [];

    workflow.nodes.forEach((node) => {
      if (node.action === "InputRouter") {
        const port = node.with.routingPort || DEFAULT_ROUTING_PORT;
        pseudoInputs.push({ nodeId: node.id, portName: port });
      } else if (node.action === "OutputRouter") {
        const port = node.with.routingPort || DEFAULT_ROUTING_PORT;
        pseudoOutputs.push({ nodeId: node.id, portName: port });
      }
    });

    return { pseudoInputs, pseudoOutputs };
  };

  const workflowsPromises = graphs.map(async (graph: EngineReadyGraph) => {
    const nodes = (
      await convertNodes(graph.nodes, getSubworkflowPseudoPorts)
    ).filter(isDefined);

    const edges = convertEdges(graph.edges);

    const { nodes: layoutedNodes, edges: layoutedEdges } = autoLayout(
      layoutType ?? "dagre",
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
