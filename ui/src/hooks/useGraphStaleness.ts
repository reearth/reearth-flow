import { useEffect, useRef } from "react";
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
      // Format: "edgeId:source→target"
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

  // Apply stored subworkflow bridge links.
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

      // internalOutputRouter → subworkflowNode (propagate out)
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
  while (queue.length > 0) {
    const current = queue.shift();
    if (!current || visited.has(current)) continue;
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
  const removedEdgeTargets = new Set<string>();

  const deletedNodeIds = new Set<string>();

  for (const wfId of Object.keys(snapshot.nodeHashes)) {
    const snapHashes = snapshot.nodeHashes[wfId] ?? {};
    const currHashes = current.nodeHashes[wfId] ?? {};
    const currNodeIds = new Set(current.nodeIds[wfId] ?? []);

    for (const nodeId of Object.keys(snapHashes)) {
      if (!currNodeIds.has(nodeId)) {
        // Node was deleted since the snapshot.
        deletedNodeIds.add(nodeId);
      } else if (snapHashes[nodeId] !== currHashes[nodeId]) {
        // Params or isDisabled changed.
        changedNodeIds.add(nodeId);
      }
    }

    // Edges removed
    const snapSigs = new Set(snapshot.edgeSignatures[wfId] ?? []);
    const currSigs = new Set(current.edgeSignatures[wfId] ?? []);
    for (const sig of snapSigs) {
      if (!currSigs.has(sig)) {
        const colonIdx = sig.indexOf(":");
        const rest = sig.slice(colonIdx + 1);
        const arrowIdx = rest.indexOf("→");
        if (arrowIdx !== -1) removedEdgeTargets.add(rest.slice(arrowIdx + 1));
      }
    }
  }

  const adj = buildAdjacency(yWorkflows);
  const staleIds = new Set<string>();

  for (const id of [...changedNodeIds, ...removedEdgeTargets]) {
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

export default function useGraphStaleness({
  yWorkflows,
  undoManager,
}: {
  yWorkflows: Y.Map<YWorkflow>;
  undoManager: Y.UndoManager | null;
}) {
  const [currentProject] = useCurrentProject();
  const { value: debugRunState, updateValue } = useIndexedDB("debugRun");

  // Refs so observeDeep / undo callbacks always see latest values without
  // needing to re-subscribe whenever they change.
  const debugJobRef = useRef<JobState | undefined>(undefined);
  const updateValueRef = useRef(updateValue);
  const projectIdRef = useRef(currentProject?.id);
  const undoManagerRef = useRef(undoManager);

  useEffect(() => {
    const job = debugRunState?.jobs?.find(
      (j) => j.projectId === currentProject?.id,
    );
    debugJobRef.current = job as JobState | undefined;
    updateValueRef.current = updateValue;
    projectIdRef.current = currentProject?.id;
    undoManagerRef.current = undoManager;
  });

  // Track the previous job id to detect when a new run starts.
  const prevJobIdRef = useRef<string | undefined>(undefined);
  const debugJob = debugRunState?.jobs?.find(
    (j) => j.projectId === currentProject?.id,
  );

  // Capture a fresh snapshot whenever a new debug job begins.
  useEffect(() => {
    const jobId = debugJob?.jobId;
    if (!jobId || jobId === prevJobIdRef.current) return;
    prevJobIdRef.current = jobId;

    const snapshot = captureSnapshot(yWorkflows);
    updateValue((prev) => ({
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
  }, [debugJob?.jobId, yWorkflows, currentProject?.id, updateValue]);

  // Deep-observe all workflow changes and update staleness accordingly.
  useEffect(() => {
    const handleGraphChange = (events: Y.YEvent<any>[]) => {
      const job = debugJobRef.current;
      if (!job?.jobId || !job.graphSnapshot) return;
      // Don't flag stale while a run is actively in progress.
      if (job.status === "running" || job.status === "queued") return;

      // Skip events that originated from an undo/redo — handled separately.
      const um = undoManagerRef.current;
      if (um && events.some((e) => e.transaction.origin === um)) return;

      const currentSnapshot = captureSnapshot(yWorkflows);
      if (!isSnapshotDifferent(job.graphSnapshot, currentSnapshot)) {
        // Graph matches the run-time snapshot — clear any stale flags that
        // were set by a previous change that has since been manually reverted.
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

      // Classify events to determine which nodes have stale downstream data.
      const changedParamNodeIds = new Set<string>();
      const deletedNodeIds = new Set<string>();
      const removedEdgeTargets = new Set<string>();

      for (const event of events) {
        const path = event.path as string[];

        if (path.length >= 2 && path[1] === "nodes") {
          if (path.length === 2) {
            event.changes.keys.forEach((change, nodeId) => {
              if (change.action === "delete") deletedNodeIds.add(nodeId);
            });
          } else if (path.length >= 4 && path[3] === "data") {
            // Node data map changed — check if params or isDisabled was the key.
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
          event.changes.keys.forEach((change, _edgeId) => {
            if (change.action === "delete") {
              const oldEdge = change.oldValue as Y.Map<any> | undefined;
              const target = (
                oldEdge?.get("target") as Y.Text | undefined
              )?.toString();
              if (target) removedEdgeTargets.add(target);
            }
          });
        }
      }

      const adj = buildAdjacency(yWorkflows);
      const staleIds = new Set<string>();

      for (const id of [...changedParamNodeIds, ...removedEdgeTargets]) {
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
    return () => yWorkflows.unobserveDeep(handleGraphChange);
  }, [yWorkflows]);

  // After undo/redo, re-diff against the snapshot to potentially clear staleness.
  useEffect(() => {
    if (!undoManager) return;

    const handleUndoRedo = () => {
      const job = debugJobRef.current;
      if (!job?.jobId || !job.graphSnapshot) return;

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
}
