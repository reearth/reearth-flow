import { useEffect, useMemo, useRef } from "react";
import * as Y from "yjs";

import { useIndexedDB } from "@flow/lib/indexedDB";
import type { YEdgesMap, YNodesMap, YWorkflow } from "@flow/lib/yjs/types";
import { GraphSnapshot, JobState, useCurrentProject } from "@flow/stores";

function sortedJsonStringify(obj: unknown): string {
  return JSON.stringify(obj, (_, value) => {
    if (value !== null && typeof value === "object" && !Array.isArray(value)) {
      return Object.keys(value as object)
        .sort()
        .reduce(
          (sorted: Record<string, unknown>, key) => {
            sorted[key] = (value as Record<string, unknown>)[key];
            return sorted;
          },
          {} as Record<string, unknown>,
        );
    }
    return value;
  });
}

function captureSnapshot(yWorkflows: Y.Map<YWorkflow>): GraphSnapshot {
  const nodeHashes: GraphSnapshot["nodeHashes"] = {};
  const nodeIds: GraphSnapshot["nodeIds"] = {};
  const edgeSignatures: GraphSnapshot["edgeSignatures"] = {};
  const subworkflowBridges: GraphSnapshot["subworkflowBridges"] = [];

  yWorkflows.forEach((yWorkflow, workflowId) => {
    const yNodes = yWorkflow.get("nodes") as YNodesMap | undefined;
    const yEdges = yWorkflow.get("edges") as YEdgesMap | undefined;

    nodeHashes[workflowId] = {};
    const wfNodeIds: string[] = [];

    yNodes?.forEach((yNode, nodeId) => {
      wfNodeIds.push(nodeId);
      const yData = yNode.get("data") as Y.Map<any> | undefined;
      const params = yData?.get("params") ?? {};
      const isDisabled = yData?.get("isDisabled") ?? false;
      nodeHashes[workflowId][nodeId] = sortedJsonStringify({
        params,
        isDisabled,
      });

      const isSubworkflow = !!(
        yData?.get("subworkflowId") as Y.Text | undefined
      )?.toString();
      if (isSubworkflow) {
        const pseudoOutputs = yData?.get("pseudoOutputs") as
          | Y.Array<Y.Map<any>>
          | undefined;
        pseudoOutputs?.forEach((po) => {
          const routerId = (po.get("nodeId") as Y.Text | undefined)?.toString();
          if (routerId) subworkflowBridges.push([routerId, nodeId]);
        });

        const pseudoInputs = yData?.get("pseudoInputs") as
          | Y.Array<Y.Map<any>>
          | undefined;
        pseudoInputs?.forEach((pi) => {
          const routerId = (pi.get("nodeId") as Y.Text | undefined)?.toString();
          if (routerId) subworkflowBridges.push([nodeId, routerId]);
        });
      }
    });

    nodeIds[workflowId] = wfNodeIds.sort();

    const sigs: string[] = [];
    yEdges?.forEach((yEdge, edgeId) => {
      const source =
        (yEdge.get("source") as Y.Text | undefined)?.toString() ?? "";
      const target =
        (yEdge.get("target") as Y.Text | undefined)?.toString() ?? "";
      sigs.push(`${edgeId}:${source}→${target}`);
    });
    edgeSignatures[workflowId] = sigs.sort();
  });

  return { nodeHashes, nodeIds, edgeSignatures, subworkflowBridges };
}

function isSnapshotDifferent(a: GraphSnapshot, b: GraphSnapshot): boolean {
  const aWfIds = Object.keys(a.nodeIds).sort();
  const bWfIds = Object.keys(b.nodeIds).sort();
  if (JSON.stringify(aWfIds) !== JSON.stringify(bWfIds)) return true;

  for (const wfId of aWfIds) {
    if (JSON.stringify(a.nodeIds[wfId]) !== JSON.stringify(b.nodeIds[wfId]))
      return true;
    if (
      JSON.stringify(a.edgeSignatures[wfId]) !==
      JSON.stringify(b.edgeSignatures[wfId])
    )
      return true;
    const aHashes = a.nodeHashes[wfId] ?? {};
    const bHashes = b.nodeHashes[wfId] ?? {};
    for (const nodeId of a.nodeIds[wfId]) {
      if (aHashes[nodeId] !== bHashes[nodeId]) return true;
    }
  }
  return false;
}

function buildAdjacencyFromSnapshot(
  snapshot: GraphSnapshot,
): Map<string, Set<string>> {
  const adj = new Map<string, Set<string>>();

  for (const wfId of Object.keys(snapshot.edgeSignatures)) {
    for (const sig of snapshot.edgeSignatures[wfId]) {
      const colonIdx = sig.indexOf(":");
      const rest = sig.slice(colonIdx + 1);
      const arrowIdx = rest.indexOf("→");
      if (arrowIdx === -1) continue;
      const source = rest.slice(0, arrowIdx);
      const target = rest.slice(arrowIdx + 1);
      if (source && target) {
        if (!adj.has(source)) adj.set(source, new Set());
        adj.get(source)?.add(target);
      }
    }
  }

  for (const [from, to] of snapshot.subworkflowBridges ?? []) {
    if (!adj.has(from)) adj.set(from, new Set());
    adj.get(from)?.add(to);
  }

  return adj;
}

function buildAdjacency(
  yWorkflows: Y.Map<YWorkflow>,
): Map<string, Set<string>> {
  const adj = new Map<string, Set<string>>();

  yWorkflows.forEach((yWorkflow) => {
    const yEdges = yWorkflow.get("edges") as YEdgesMap | undefined;
    yEdges?.forEach((yEdge) => {
      const source = (yEdge.get("source") as Y.Text | undefined)?.toString();
      const target = (yEdge.get("target") as Y.Text | undefined)?.toString();
      if (source && target) {
        if (!adj.has(source)) adj.set(source, new Set());
        adj.get(source)?.add(target);
      }
    });

    const yNodes = yWorkflow.get("nodes") as YNodesMap | undefined;
    yNodes?.forEach((yNode, nodeId) => {
      const yData = yNode.get("data") as Y.Map<any> | undefined;
      if (!(yData?.get("subworkflowId") as Y.Text | undefined)?.toString())
        return;

      const pseudoOutputs = yData?.get("pseudoOutputs") as
        | Y.Array<Y.Map<any>>
        | undefined;
      pseudoOutputs?.forEach((po) => {
        const routerId = (po.get("nodeId") as Y.Text | undefined)?.toString();
        if (routerId) {
          if (!adj.has(routerId)) adj.set(routerId, new Set());
          adj.get(routerId)?.add(nodeId);
        }
      });

      const pseudoInputs = yData?.get("pseudoInputs") as
        | Y.Array<Y.Map<any>>
        | undefined;
      pseudoInputs?.forEach((pi) => {
        const routerId = (pi.get("nodeId") as Y.Text | undefined)?.toString();
        if (routerId) {
          if (!adj.has(nodeId)) adj.set(nodeId, new Set());
          adj.get(nodeId)?.add(routerId);
        }
      });
    });
  });

  return adj;
}

function bfsDownstream(
  startId: string,
  adj: Map<string, Set<string>>,
): Set<string> {
  const visited = new Set<string>();
  const queue = [startId];
  let i = 0;
  while (i < queue.length) {
    const current = queue[i++];
    if (visited.has(current)) continue;
    visited.add(current);
    adj.get(current)?.forEach((next) => queue.push(next));
  }
  return visited;
}

function computeStaleNodesFromDiff(
  snapshot: GraphSnapshot,
  current: GraphSnapshot,
  yWorkflows: Y.Map<YWorkflow>,
): string[] {
  const changedNodeIds = new Set<string>();
  const changedEdgeTargets = new Set<string>();

  const deletedNodeIds = new Set<string>();

  const extractTarget = (sig: string): string | undefined => {
    const rest = sig.slice(sig.indexOf(":") + 1);
    const arrowIdx = rest.indexOf("→");
    return arrowIdx !== -1 ? rest.slice(arrowIdx + 1) : undefined;
  };

  for (const wfId of Object.keys(snapshot.nodeHashes)) {
    const snapHashes = snapshot.nodeHashes[wfId] ?? {};
    const currHashes = current.nodeHashes[wfId] ?? {};
    const currNodeIds = new Set(current.nodeIds[wfId] ?? []);

    for (const nodeId of Object.keys(snapHashes)) {
      if (!currNodeIds.has(nodeId)) {
        deletedNodeIds.add(nodeId);
      } else if (snapHashes[nodeId] !== currHashes[nodeId]) {
        changedNodeIds.add(nodeId);
      }
    }

    const snapSigs = new Set(snapshot.edgeSignatures[wfId] ?? []);
    const currSigs = new Set(current.edgeSignatures[wfId] ?? []);

    for (const sig of snapSigs) {
      if (!currSigs.has(sig)) {
        const target = extractTarget(sig);
        if (target) changedEdgeTargets.add(target);
      }
    }
    for (const sig of currSigs) {
      if (!snapSigs.has(sig)) {
        const target = extractTarget(sig);
        if (target) changedEdgeTargets.add(target);
      }
    }
  }

  const adj = buildAdjacency(yWorkflows);
  const staleIds = new Set<string>();

  for (const id of [...changedNodeIds, ...changedEdgeTargets]) {
    bfsDownstream(id, adj).forEach((n) => staleIds.add(n));
  }

  if (deletedNodeIds.size > 0) {
    const oldAdj = buildAdjacencyFromSnapshot(snapshot);
    for (const nodeId of deletedNodeIds) {
      bfsDownstream(nodeId, oldAdj).forEach((n) => {
        if (!deletedNodeIds.has(n)) staleIds.add(n);
      });
    }
  }

  return Array.from(staleIds);
}

function isRelevantEvent(event: Y.YEvent<any>): boolean {
  const path = event.path as string[];
  if (path[1] === "edges") return true;
  if (path[1] === "nodes") {
    if (path.length === 2) return true;
    if (path.length >= 4 && path[3] === "data") {
      let relevant = false;
      event.changes.keys.forEach((_, key) => {
        if (key === "params" || key === "isDisabled") relevant = true;
      });
      return relevant;
    }
    return false;
  }
  return false;
}

export default function useGraphStaleness({
  yWorkflows,
  undoManager,
}: {
  yWorkflows: Y.Map<YWorkflow>;
  undoManager: Y.UndoManager | null;
}) {
  const [currentProject] = useCurrentProject();
  const { value: debugRunState, updateValue } = useIndexedDB("debugRun");

  const debugJobRef = useRef<JobState | undefined>(undefined);
  const updateValueRef = useRef(updateValue);
  const projectIdRef = useRef(currentProject?.id);
  const undoManagerRef = useRef(undoManager);
  const adjacencyRef = useRef<Map<string, Set<string>> | null>(null);

  useEffect(() => {
    const job = debugRunState?.jobs?.find(
      (j) => j.projectId === currentProject?.id,
    );
    debugJobRef.current = job as JobState | undefined;
    updateValueRef.current = updateValue;
    projectIdRef.current = currentProject?.id;
    undoManagerRef.current = undoManager;
  });

  const prevJobIdRef = useRef<string | undefined>(undefined);
  const debugJob = debugRunState?.jobs?.find(
    (j) => j.projectId === currentProject?.id,
  ) as JobState | undefined;

  useEffect(() => {
    const jobId = debugJob?.jobId;

    if (!jobId) {
      prevJobIdRef.current = undefined;
      return;
    }

    if (jobId === prevJobIdRef.current) return;
    prevJobIdRef.current = jobId;

    const snapshot = captureSnapshot(yWorkflows);
    updateValueRef.current((prev) => ({
      ...prev,
      jobs: (prev.jobs ?? []).map((j) => {
        if (j.projectId !== currentProject?.id) return j;
        return {
          ...j,
          graphSnapshot: snapshot,
          isRunStale: false,
          staleNodeIds: [],
        };
      }),
    }));
  }, [debugJob?.jobId, yWorkflows, currentProject?.id]);

  useEffect(() => {
    adjacencyRef.current = null;

    const handleGraphChange = (events: Y.YEvent<any>[]) => {
      const job = debugJobRef.current;
      if (!job?.jobId || !job.graphSnapshot) return;
      if (job.status === "running" || job.status === "queued") return;

      const um = undoManagerRef.current;
      if (um && events.some((e) => e.transaction.origin === um)) return;

      if (!events.some(isRelevantEvent)) return;

      const currentSnapshot = captureSnapshot(yWorkflows);
      if (!isSnapshotDifferent(job.graphSnapshot, currentSnapshot)) {
        if (job.isRunStale) {
          const projectId = projectIdRef.current;
          updateValueRef.current((prev) => ({
            ...prev,
            jobs: (prev.jobs ?? []).map((j) => {
              if (j.projectId !== projectId) return j;
              return { ...j, isRunStale: false, staleNodeIds: [] };
            }),
          }));
        }
        return;
      }

      const changedParamNodeIds = new Set<string>();
      const deletedNodeIds = new Set<string>();
      const changedEdgeTargets = new Set<string>();

      for (const event of events) {
        const path = event.path as string[];

        if (path.length >= 2 && path[1] === "nodes") {
          if (path.length === 2) {
            event.changes.keys.forEach((change, nodeId) => {
              if (change.action === "delete") deletedNodeIds.add(nodeId);
            });
          } else if (path.length >= 4 && path[3] === "data") {
            const nodeId = path[2];
            event.changes.keys.forEach((change, key) => {
              if (
                (key === "params" || key === "isDisabled") &&
                change.action !== "delete"
              ) {
                changedParamNodeIds.add(nodeId);
              }
            });
          }
        } else if (
          path.length >= 2 &&
          path[1] === "edges" &&
          path.length === 2
        ) {
          const workflowId = path[0];
          const yEdges = yWorkflows.get(workflowId)?.get("edges") as
            | YEdgesMap
            | undefined;

          event.changes.keys.forEach((change, edgeId) => {
            if (change.action === "delete") {
              const oldEdge = change.oldValue as Y.Map<any> | undefined;
              const target = (
                oldEdge?.get("target") as Y.Text | undefined
              )?.toString();
              if (target) changedEdgeTargets.add(target);
            } else if (change.action === "add") {
              const target = (
                yEdges?.get(edgeId)?.get("target") as Y.Text | undefined
              )?.toString();
              if (target) changedEdgeTargets.add(target);
            } else if (change.action === "update") {
              const oldEdge = change.oldValue as Y.Map<any> | undefined;
              const oldTarget = (
                oldEdge?.get("target") as Y.Text | undefined
              )?.toString();
              if (oldTarget) changedEdgeTargets.add(oldTarget);
              const newTarget = (
                yEdges?.get(edgeId)?.get("target") as Y.Text | undefined
              )?.toString();
              if (newTarget) changedEdgeTargets.add(newTarget);
            }
          });
        }
      }

      const affectsTopology = events.some((e) => {
        const p = e.path as string[];
        return (
          (p[1] === "edges" && p.length === 2) ||
          (p[1] === "nodes" && p.length === 2)
        );
      });
      if (affectsTopology || !adjacencyRef.current) {
        adjacencyRef.current = buildAdjacency(yWorkflows);
      }
      const adj = adjacencyRef.current;
      const staleIds = new Set<string>();

      for (const id of [...changedParamNodeIds, ...changedEdgeTargets]) {
        bfsDownstream(id, adj).forEach((n) => staleIds.add(n));
      }

      if (deletedNodeIds.size > 0) {
        const oldAdj = buildAdjacencyFromSnapshot(job.graphSnapshot);
        for (const nodeId of deletedNodeIds) {
          bfsDownstream(nodeId, oldAdj).forEach((n) => {
            if (!deletedNodeIds.has(n)) staleIds.add(n);
          });
        }
      }

      const projectId = projectIdRef.current;
      updateValueRef.current((prev) => ({
        ...prev,
        jobs: (prev.jobs ?? []).map((j) => {
          if (j.projectId !== projectId) return j;
          const merged = Array.from(
            new Set([...(j.staleNodeIds ?? []), ...staleIds]),
          );
          return { ...j, isRunStale: true, staleNodeIds: merged };
        }),
      }));
    };

    yWorkflows.observeDeep(handleGraphChange);
    return () => {
      yWorkflows.unobserveDeep(handleGraphChange);
      adjacencyRef.current = null;
    };
  }, [yWorkflows]);

  useEffect(() => {
    if (!undoManager) return;

    const handleUndoRedo = () => {
      const job = debugJobRef.current;
      if (!job?.jobId || !job.graphSnapshot) return;

      adjacencyRef.current = null;

      const currentSnapshot = captureSnapshot(yWorkflows);
      const projectId = projectIdRef.current;

      if (!isSnapshotDifferent(job.graphSnapshot, currentSnapshot)) {
        updateValueRef.current((prev) => ({
          ...prev,
          jobs: (prev.jobs ?? []).map((j) => {
            if (j.projectId !== projectId) return j;
            return { ...j, isRunStale: false, staleNodeIds: [] };
          }),
        }));
      } else {
        const staleNodeIds = computeStaleNodesFromDiff(
          job.graphSnapshot,
          currentSnapshot,
          yWorkflows,
        );
        updateValueRef.current((prev) => ({
          ...prev,
          jobs: (prev.jobs ?? []).map((j) => {
            if (j.projectId !== projectId) return j;
            return {
              ...j,
              isRunStale: true,
              staleNodeIds,
            };
          }),
        }));
      }
    };

    undoManager.on("stack-item-popped", handleUndoRedo);
    return () => undoManager.off("stack-item-popped", handleUndoRedo);
  }, [undoManager, yWorkflows]);

  const staleIdsKey = debugJob?.isRunStale
    ? (debugJob.staleNodeIds ?? []).slice().sort().join("\0")
    : "";

  const staleNodeIds = useMemo(
    () =>
      staleIdsKey
        ? new Set<string>(staleIdsKey.split("\0"))
        : new Set<string>(),
    [staleIdsKey],
  );

  return { staleNodeIds };
}
