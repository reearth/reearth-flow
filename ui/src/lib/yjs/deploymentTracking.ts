import type * as Y from "yjs";

import type { YEdgesMap, YNode, YNodesMap, YWorkflow } from "./types";

// Canvas-only node types. They never reach the engine, so adding, removing or
// editing them does not change what a deployment runs.
const IGNORED_NODE_TYPES = new Set(["note", "batch"]);

const DATA_KEYS = ["params", "isDisabled"] as const;

const nodeType = (yNode?: YNode) =>
  (yNode?.get("type") as Y.Text | undefined)?.toString();

const isTrackedNode = (yNode?: YNode) => {
  const type = nodeType(yNode);
  return !!type && !IGNORED_NODE_TYPES.has(type);
};

// IDs of every note and batch node, so their deletion can be told apart from a
// real node's. A deleted Y.Map's contents are already gone inside the observer
// (oldValue reads as empty), so the type has to be known beforehand.
export function collectIgnoredNodeIds(
  yWorkflows: Y.Map<YWorkflow>,
): Set<string> {
  const ids = new Set<string>();
  yWorkflows.forEach((yWorkflow) => {
    (yWorkflow.get("nodes") as YNodesMap | undefined)?.forEach(
      (yNode, nodeId) => {
        if (!isTrackedNode(yNode)) ids.add(nodeId);
      },
    );
  });
  return ids;
}

// Whether a Yjs event on the workflows map changes what a deployment would
// run: a tracked node added or removed, any edge change, or a tracked node's
// params or disabled state. Positions and every other field are ignored.
//
// ignoredNodeIds must hold the note and batch nodes that existed before the
// event. Added ones are recorded into it, so it stays current.
export function isDeploymentRelevantEvent(
  event: Y.YEvent<any>,
  yWorkflows: Y.Map<YWorkflow>,
  ignoredNodeIds: Set<string>,
): boolean {
  const path = event.path as string[];

  if (path[1] === "edges") return true;
  if (path[1] !== "nodes") return false;

  const yNodes = yWorkflows.get(path[0])?.get("nodes") as YNodesMap | undefined;

  if (path.length === 2) {
    let relevant = false;
    event.changes.keys.forEach((change, nodeId) => {
      if (change.action === "delete") {
        if (!ignoredNodeIds.delete(nodeId)) relevant = true;
        return;
      }
      if (isTrackedNode(yNodes?.get(nodeId))) {
        relevant = true;
      } else {
        ignoredNodeIds.add(nodeId);
      }
    });
    return relevant;
  }

  if (path.length === 4 && path[3] === "data") {
    if (!isTrackedNode(yNodes?.get(path[2]))) return false;
    let relevant = false;
    event.changes.keys.forEach((_, key) => {
      if ((DATA_KEYS as readonly string[]).includes(key)) relevant = true;
    });
    return relevant;
  }

  return false;
}

// Y.Map doesn't guarantee key order, so keys are sorted before stringifying.
function sortedJsonStringify(value: unknown): string {
  return JSON.stringify(value, (_, v) => {
    if (v !== null && typeof v === "object" && !Array.isArray(v)) {
      return Object.fromEntries(
        Object.keys(v as object)
          .sort()
          .map((k) => [k, (v as Record<string, unknown>)[k]]),
      );
    }
    return v;
  });
}

// cyrb53: a fast 53-bit string hash. Collision resistance is not a concern
// here; it is only ever compared against the one fingerprint saved at deploy
// time.
function hash53(str: string): string {
  let h1 = 0xdeadbeef;
  let h2 = 0x41c6ce57;
  for (let i = 0; i < str.length; i++) {
    const ch = str.charCodeAt(i);
    h1 = Math.imul(h1 ^ ch, 2654435761);
    h2 = Math.imul(h2 ^ ch, 1597334677);
  }
  h1 = Math.imul(h1 ^ (h1 >>> 16), 2246822507);
  h1 ^= Math.imul(h2 ^ (h2 >>> 13), 3266489909);
  h2 = Math.imul(h2 ^ (h2 >>> 16), 2246822507);
  h2 ^= Math.imul(h1 ^ (h1 >>> 13), 3266489909);
  return (4294967296 * (2097151 & h2) + (h1 >>> 0)).toString(16);
}

const text = (value: unknown) => (value as Y.Text | undefined)?.toString();

// A fixed-size fingerprint of the parts of the graph that
// isDeploymentRelevantEvent tracks. Saved at deploy time and compared again on
// undo/redo, so undoing back to the deployed state clears the changed flag.
export function computeDeploymentFingerprint(
  yWorkflows: Y.Map<YWorkflow>,
): string {
  const workflows = Array.from(yWorkflows.entries())
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([workflowId, yWorkflow]) => {
      const nodes: string[] = [];
      (yWorkflow.get("nodes") as YNodesMap | undefined)?.forEach(
        (yNode, nodeId) => {
          if (!isTrackedNode(yNode)) return;
          const yData = yNode.get("data") as Y.Map<unknown> | undefined;
          nodes.push(
            sortedJsonStringify([
              nodeId,
              nodeType(yNode),
              text(yData?.get("officialName")),
              yData?.get("params") ?? null,
              yData?.get("isDisabled") ?? false,
            ]),
          );
        },
      );

      const edges: string[] = [];
      (yWorkflow.get("edges") as YEdgesMap | undefined)?.forEach(
        (yEdge, edgeId) => {
          edges.push(
            [
              edgeId,
              text(yEdge.get("source")),
              text(yEdge.get("sourceHandle")),
              text(yEdge.get("target")),
              text(yEdge.get("targetHandle")),
            ].join("|"),
          );
        },
      );

      return [workflowId, nodes.sort(), edges.sort()];
    });

  return hash53(JSON.stringify(workflows));
}
