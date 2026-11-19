import {
  DEFAULT_EDGE_PORT,
  DEFAULT_ENTRY_GRAPH_ID,
} from "@flow/global-constants";
import { deployableNodeTypes } from "@flow/types";
import type { Edge, Node, Workflow } from "@flow/types";

type RouterPort = { subworkflowNodeId: string; portName: string };

export type RunGraph = {
  nodes: Map<string, Node>;
  workflowOf: Map<string, string>;
  subworkflowNodeOf: Map<string, string>;
  incoming: Map<string, Edge[]>;
  outgoing: Map<string, Edge[]>;
  inputRouters: Map<string, RouterPort>;
  outputRouters: Map<string, RouterPort>;
  inputRouterAt: Map<string, string[]>;
  outputRouterAt: Map<string, string[]>;
};

const portKey = (nodeId: string, portName: string) => `${nodeId}\0${portName}`;

const sourcePort = (edge: Edge) => edge.sourceHandle ?? DEFAULT_EDGE_PORT;
const targetPort = (edge: Edge) => edge.targetHandle ?? DEFAULT_EDGE_PORT;

const isRunnable = (node: Node) =>
  deployableNodeTypes.includes(node.type) && !node.data.isDisabled;

const append = <T>(map: Map<string, T[]>, key: string, value: T) => {
  const list = map.get(key);
  if (list) list.push(value);
  else map.set(key, [value]);
};

export const buildRunGraph = (workflows: Workflow[]): RunGraph => {
  const graph: RunGraph = {
    nodes: new Map(),
    workflowOf: new Map(),
    subworkflowNodeOf: new Map(),
    incoming: new Map(),
    outgoing: new Map(),
    inputRouters: new Map(),
    outputRouters: new Map(),
    inputRouterAt: new Map(),
    outputRouterAt: new Map(),
  };

  workflows.forEach((workflow) =>
    workflow.nodes?.forEach((node) => {
      if (!isRunnable(node)) return;
      graph.nodes.set(node.id, node);
      graph.workflowOf.set(node.id, workflow.id);
      if (node.type !== "subworkflow" || !node.data.subworkflowId) return;
      graph.subworkflowNodeOf.set(node.data.subworkflowId, node.id);
      node.data.pseudoInputs?.forEach(({ nodeId, portName }) => {
        graph.inputRouters.set(nodeId, {
          subworkflowNodeId: node.id,
          portName,
        });
        append(graph.inputRouterAt, portKey(node.id, portName), nodeId);
      });
      node.data.pseudoOutputs?.forEach(({ nodeId, portName }) => {
        graph.outputRouters.set(nodeId, {
          subworkflowNodeId: node.id,
          portName,
        });
        append(graph.outputRouterAt, portKey(node.id, portName), nodeId);
      });
    }),
  );

  workflows.forEach((workflow) =>
    workflow.edges?.forEach((edge) => {
      if (!graph.nodes.has(edge.source) || !graph.nodes.has(edge.target))
        return;
      append(graph.outgoing, edge.source, edge);
      append(graph.incoming, edge.target, edge);
    }),
  );

  return graph;
};

const isSubworkflowNode = (graph: RunGraph, nodeId: string) =>
  graph.nodes.get(nodeId)?.type === "subworkflow";

const resolveSources = (graph: RunGraph, edge: Edge): string[] =>
  isSubworkflowNode(graph, edge.source)
    ? (graph.outputRouterAt.get(portKey(edge.source, sourcePort(edge))) ?? [])
    : [edge.source];

const resolveTargets = (graph: RunGraph, edge: Edge): string[] =>
  isSubworkflowNode(graph, edge.target)
    ? (graph.inputRouterAt.get(portKey(edge.target, targetPort(edge))) ?? [])
    : [edge.target];

export const feedersOf = (graph: RunGraph, nodeId: string): string[] => {
  const router = graph.inputRouters.get(nodeId);
  const edges = router
    ? (graph.incoming.get(router.subworkflowNodeId) ?? []).filter(
        (edge) => targetPort(edge) === router.portName,
      )
    : (graph.incoming.get(nodeId) ?? []);
  return edges.flatMap((edge) => resolveSources(graph, edge));
};

const consumersOf = (graph: RunGraph, nodeId: string): string[] => {
  const router = graph.outputRouters.get(nodeId);
  const edges = router
    ? (graph.outgoing.get(router.subworkflowNodeId) ?? []).filter(
        (edge) => sourcePort(edge) === router.portName,
      )
    : (graph.outgoing.get(nodeId) ?? []);
  return edges.flatMap((edge) => resolveTargets(graph, edge));
};

const walk = (
  graph: RunGraph,
  startId: string,
  entry: (subworkflowNode: Node) => string[],
  next: (graph: RunGraph, nodeId: string) => string[],
): Set<string> => {
  const visited = new Set<string>();
  const start = graph.nodes.get(startId);
  if (!start) return visited;

  visited.add(startId);
  const queue =
    start.type === "subworkflow" ? entry(start) : next(graph, startId);
  for (const id of queue) {
    if (visited.has(id) || !graph.nodes.has(id)) continue;
    visited.add(id);
    queue.push(...next(graph, id));
  }
  return visited;
};

export const upstreamOf = (graph: RunGraph, nodeId: string) =>
  walk(
    graph,
    nodeId,
    (node) => node.data.pseudoOutputs?.map((port) => port.nodeId) ?? [],
    feedersOf,
  );

export const downstreamOf = (graph: RunGraph, nodeId: string) =>
  walk(
    graph,
    nodeId,
    (node) => node.data.pseudoInputs?.map((port) => port.nodeId) ?? [],
    consumersOf,
  );

const withContainers = (graph: RunGraph, nodeIds: Set<string>) => {
  const result = new Set(nodeIds);
  nodeIds.forEach((id) => {
    let workflowId = graph.workflowOf.get(id);
    while (workflowId && workflowId !== DEFAULT_ENTRY_GRAPH_ID) {
      const container = graph.subworkflowNodeOf.get(workflowId);
      if (!container || result.has(container)) break;
      result.add(container);
      workflowId = graph.workflowOf.get(container);
    }
  });
  return result;
};

export const trimWorkflowsToNode = (
  workflows: Workflow[],
  targetNodeId: string,
): { workflows: Workflow[]; keptNodeIds: Set<string> } | undefined => {
  const graph = buildRunGraph(workflows);
  if (!graph.nodes.has(targetNodeId)) return undefined;

  const keptNodeIds = withContainers(graph, upstreamOf(graph, targetNodeId));

  const keepsEdge = (edge: Edge) => {
    if (!keptNodeIds.has(edge.source) || !keptNodeIds.has(edge.target))
      return false;
    return (
      resolveSources(graph, edge).some((id) => keptNodeIds.has(id)) &&
      resolveTargets(graph, edge).some((id) => keptNodeIds.has(id))
    );
  };

  const trimmed = workflows
    .filter((workflow) => {
      if (workflow.id === DEFAULT_ENTRY_GRAPH_ID) return true;
      const container = graph.subworkflowNodeOf.get(workflow.id);
      return !!container && keptNodeIds.has(container);
    })
    .map((workflow) => ({
      ...workflow,
      nodes: workflow.nodes?.filter((node) => keptNodeIds.has(node.id)),
      edges: workflow.edges?.filter(keepsEdge),
    }));

  return { workflows: trimmed, keptNodeIds };
};

export const runnableNodeIds = (workflows: Workflow[]): string[] =>
  Array.from(buildRunGraph(workflows).nodes.keys());

export const lastRunCovers = (
  graph: RunGraph,
  ranNodeIds: readonly string[],
  startNodeId: string,
): boolean => {
  const executed = downstreamOf(graph, startNodeId);
  const ran = new Set(ranNodeIds);
  for (const id of executed) {
    for (const feeder of feedersOf(graph, id)) {
      if (!executed.has(feeder) && !ran.has(feeder)) return false;
    }
  }
  return true;
};
