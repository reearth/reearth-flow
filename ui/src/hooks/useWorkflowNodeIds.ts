import { useEffect, useState } from "react";
import type * as Y from "yjs";

import type { YNodesMap, YWorkflow } from "@flow/lib/yjs/types";

const collect = (yWorkflows?: Y.Map<YWorkflow> | null): string[] => {
  if (!yWorkflows) return [];
  const ids: string[] = [];
  // Every workflow, not just the open one: a subworkflow's nodes report
  // diagnostics of their own, and skipping them would hide those silently.
  yWorkflows.forEach((yWorkflow) => {
    const yNodes = yWorkflow.get("nodes") as YNodesMap | undefined;
    yNodes?.forEach((_node, nodeId) => ids.push(nodeId));
  });
  // Sorted because this array is part of a query key: an order change would
  // hash differently and refetch for nothing.
  return ids.sort();
};

const sameIds = (a: string[], b: string[]) =>
  a.length === b.length && a.every((id, i) => id === b[i]);

/**
 * Every node id across the project's workflows.
 *
 * Diagnostics are bucketed per node and the API has no job-wide query, so the
 * ids are what make a run's diagnostics reachable at all — see
 * `nodeDiagnosticsBatch`.
 *
 * `observeDeep` fires on any graph edit, including parameter changes that leave
 * the id set alone, so the result is compared before being stored: this feeds a
 * query key, and a fresh array on every keystroke would refetch continuously.
 */
export default (yWorkflows?: Y.Map<YWorkflow> | null) => {
  const [nodeIds, setNodeIds] = useState<string[]>(() => collect(yWorkflows));

  useEffect(() => {
    if (!yWorkflows) return;

    const update = () =>
      setNodeIds((previous) => {
        const next = collect(yWorkflows);
        return sameIds(previous, next) ? previous : next;
      });

    update();
    yWorkflows.observeDeep(update);
    return () => yWorkflows.unobserveDeep(update);
  }, [yWorkflows]);

  return nodeIds;
};
