import type { Node, Workflow } from "@flow/types";

import type { AutocompleteSuggestion } from "../components/ParamsDialog/components/ValueEditorDialog/components/constants";

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
 * Build a reverse-adjacency map (node id -> ids of nodes feeding into it)
 * spanning every workflow graph, including across subworkflow boundaries.
 *
 * Subworkflow connectivity is expressed on the subworkflow node via
 * pseudoInputs / pseudoOutputs, which directly name the child graph's router
 * nodes — so no port matching is needed:
 *  - pseudoInputs:  an InputRouter inside the child exposes the subworkflow's
 *    input, so its upstream is whatever feeds the subworkflow node in the
 *    parent. We link the router straight to those parent feeders (bypassing the
 *    subworkflow node) so a reader on the subworkflow's OUTPUT side never leaks
 *    onto its INPUT side through the single shared boundary node.
 *  - pseudoOutputs: an OutputRouter inside the child feeds the subworkflow
 *    node's output, so the subworkflow node's upstream is that router.
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
 * Build attribute-name autocomplete suggestions for a FlowExpr field on
 * `targetNodeId`, sourced from the probed schemas of the reader nodes UPSTREAM
 * of it — traversing edges back to their sources across subworkflow boundaries.
 * Reader attributes flow downstream, so only readers a node is actually
 * connected to below contribute, not every reader on the canvas.
 *
 * Without a target node id, returns no suggestions (a FlowExpr field is always
 * edited in the context of a node).
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

  // Reverse BFS from the target (inclusive) across all graphs.
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
