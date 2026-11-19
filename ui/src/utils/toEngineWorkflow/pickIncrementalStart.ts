import type { JobState } from "@flow/stores";
import type { AnyWorkflowVariable, Workflow } from "@flow/types";

import {
  buildRunGraph,
  downstreamOf,
  feedersOf,
  lastRunCovers,
} from "./trimToNode";

const runValues = (variables: AnyWorkflowVariable[] = []) =>
  JSON.stringify(
    variables
      .map((v) => [v.name, v.defaultValue] as const)
      .sort(([a], [b]) => a.localeCompare(b)),
  );

export const pickIncrementalStart = ({
  trimmedWorkflows,
  selectedNodeId,
  lastRun,
  variables,
}: {
  trimmedWorkflows: Workflow[];
  selectedNodeId: string;
  lastRun?: JobState;
  variables: AnyWorkflowVariable[];
}): { previousJobId: string; startNodeId: string } | undefined => {
  if (
    !lastRun?.jobId ||
    lastRun.status !== "completed" ||
    lastRun.joinedUserName ||
    !lastRun.ranNodeIds ||
    !lastRun.graphSnapshot ||
    runValues(lastRun.variables) !== runValues(variables)
  )
    return undefined;

  const graph = buildRunGraph(trimmedWorkflows);
  const isAction = (id: string) =>
    graph.nodes.has(id) && graph.nodes.get(id)?.type !== "subworkflow";

  const stale = new Set(
    (lastRun.isRunStale ? (lastRun.staleNodeIds ?? []) : []).filter(isAction),
  );
  const roots = [...stale].filter(
    (id) => !feedersOf(graph, id).some((feeder) => stale.has(feeder)),
  );

  const startNodeId =
    roots.length === 0
      ? selectedNodeId
      : roots.length === 1
        ? roots[0]
        : undefined;
  if (!startNodeId || !isAction(startNodeId)) return undefined;

  const executed = downstreamOf(graph, startNodeId);
  if ([...stale].some((id) => !executed.has(id))) return undefined;
  if (!lastRunCovers(graph, lastRun.ranNodeIds, startNodeId)) return undefined;

  return { previousJobId: lastRun.jobId, startNodeId };
};
