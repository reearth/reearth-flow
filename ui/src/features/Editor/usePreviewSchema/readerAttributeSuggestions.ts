import type { Node, Workflow } from "@flow/types";

import { AutocompleteSuggestion } from "../components/ParamsDialog/components/ValueEditorDialog/components/flowExprConstants";

const toSuggestion = (field: {
  name: string;
  type: string;
  presence: string;
}): AutocompleteSuggestion => ({
  label: field.name,
  insertText: field.name,
  type: "variable",
  detail: field.presence === "maybe" ? `${field.type} (optional)` : field.type,
});

const collectReaderFields = (
  node: Node,
  seen: Map<string, AutocompleteSuggestion>,
) => {
  if (node.type !== "reader") return;
  const ports = node.data.metadata?.schema?.ports;
  if (!ports) return;
  Object.values(ports).forEach((port) =>
    port.fields.forEach((field) => {
      if (seen.has(field.name)) return;
      seen.set(field.name, toSuggestion(field));
    }),
  );
};

/**
 * Reverse-adjacency map (node id -> ids of nodes feeding it) across every
 * graph. Subworkflow boundaries are crossed via the subworkflow node's
 * pseudoInputs/pseudoOutputs, which name the child's router nodes directly.
 * Input routers are linked to the subworkflow node's parent feeders rather than
 * the node itself, so an output-side reader can't leak onto the input side
 * through the single shared boundary node.
 */
const buildUpstreamMap = (workflows: Workflow[]): Map<string, Set<string>> => {
  const upstream = new Map<string, Set<string>>();
  const link = (node: string, source: string) => {
    const sources = upstream.get(node) ?? new Set<string>();
    sources.add(source);
    upstream.set(node, sources);
  };

  workflows.forEach((workflow) => {
    const edges = workflow.edges ?? [];
    edges.forEach((edge) => {
      if (edge.source && edge.target) link(edge.target, edge.source);
    });
    workflow.nodes?.forEach((node) => {
      if (node.type !== "subworkflow") return;
      const parentFeeders = edges
        .filter((edge) => edge.target === node.id && edge.source)
        .map((edge) => edge.source);
      node.data.pseudoInputs?.forEach((port) =>
        parentFeeders.forEach((feeder) => link(port.nodeId, feeder)),
      );
      node.data.pseudoOutputs?.forEach((port) => link(node.id, port.nodeId));
    });
  });

  return upstream;
};

/**
 * Attribute-name suggestions for a FlowExpr field on `targetNodeId`: the union
 * of probed schemas from readers upstream of it (so a node only sees readers it
 * is connected to below, not every reader on the canvas).
 */
export const buildReaderAttributeSuggestions = (
  workflows: Workflow[],
  targetNodeId?: string,
): AutocompleteSuggestion[] => {
  if (!targetNodeId) return [];

  const nodeById = new Map<string, Node>();
  workflows.forEach((workflow) =>
    workflow.nodes?.forEach((node) => nodeById.set(node.id, node)),
  );
  if (!nodeById.has(targetNodeId)) return [];

  const upstream = buildUpstreamMap(workflows);

  const reachable = new Set<string>();
  const queue: string[] = [targetNodeId];
  while (queue.length > 0) {
    const id = queue.shift();
    if (!id || reachable.has(id)) continue;
    reachable.add(id);
    upstream.get(id)?.forEach((source) => {
      if (!reachable.has(source)) queue.push(source);
    });
  }

  const seen = new Map<string, AutocompleteSuggestion>();
  reachable.forEach((id) => {
    const node = nodeById.get(id);
    if (node) collectReaderFields(node, seen);
  });

  return Array.from(seen.values());
};
