import { DEFAULT_EDGE_PORT } from "@flow/global-constants";
import type { Edge, EngineReadyEdge } from "@flow/types";

import { generateUUID } from "../generateUUID";

const edgeKey = (
  e: Pick<Edge, "source" | "target" | "sourceHandle" | "targetHandle">,
) =>
  `${e.source}:${e.sourceHandle ?? DEFAULT_EDGE_PORT}->${e.target}:${e.targetHandle ?? DEFAULT_EDGE_PORT}`;

const reconnectAroundDisabledNodes = (
  edges: Edge[],
  enabledNodeIds: Set<string>,
): Edge[] => {
  let currentEdges = edges.slice();

  // Dedup existing edges (optional but usually helpful)
  const seen = new Set<string>();
  currentEdges = currentEdges.filter((e) => {
    const k = edgeKey(e);
    if (seen.has(k)) return false;
    seen.add(k);
    return true;
  });

  while (true) {
    // Build adjacency maps once per iteration
    const incomingByTarget = new Map<string, Edge[]>();
    const outgoingBySource = new Map<string, Edge[]>();

    for (const e of currentEdges) {
      {
        let arr = incomingByTarget.get(e.target);
        if (!arr) {
          incomingByTarget.set(e.target, []);
          arr = incomingByTarget.get(e.target);
        }
        if (arr) {
          arr.push(e);
        }
      }
      {
        let arr = outgoingBySource.get(e.source);
        if (!arr) {
          outgoingBySource.set(e.source, []);
          arr = outgoingBySource.get(e.source);
        }
        if (arr) {
          arr.push(e);
        }
      }
    }

    // Find a disabled node that is "bypassable" (has both in & out)
    let disabledToBypass: string | null = null;
    for (const nodeId of new Set(
      currentEdges.flatMap((e) => [e.source, e.target]),
    )) {
      if (enabledNodeIds.has(nodeId)) continue;
      const ins = incomingByTarget.get(nodeId) ?? [];
      const outs = outgoingBySource.get(nodeId) ?? [];
      if (ins.length > 0 && outs.length > 0) {
        disabledToBypass = nodeId;
        break;
      }
    }

    if (!disabledToBypass) break;

    const incomingEdges = incomingByTarget.get(disabledToBypass) ?? [];
    const outgoingEdges = outgoingBySource.get(disabledToBypass) ?? [];

    // Create bypass edges (Cartesian product)
    const newEdges: Edge[] = [];
    for (const inEdge of incomingEdges) {
      for (const outEdge of outgoingEdges) {
        // Avoid trivial self-loops (optional; adjust if your engine supports them)
        if (inEdge.source === outEdge.target) continue;

        const bypass: Edge = {
          id: generateUUID(),
          source: inEdge.source,
          target: outEdge.target,
          sourceHandle: DEFAULT_EDGE_PORT,
          targetHandle: DEFAULT_EDGE_PORT,
        };

        const k = edgeKey(bypass);
        if (!seen.has(k)) {
          seen.add(k);
          newEdges.push(bypass);
        }
      }
    }

    // Remove only edges incident to the node we bypassed
    currentEdges = currentEdges.filter(
      (e) => e.source !== disabledToBypass && e.target !== disabledToBypass,
    );

    currentEdges.push(...newEdges);
  }

  // Finally, return only enabled→enabled edges
  return currentEdges.filter(
    (e) => enabledNodeIds.has(e.source) && enabledNodeIds.has(e.target),
  );
};

export const convertEdges = (enabledNodeIds: Set<string>, edges?: Edge[]) => {
  if (!edges) return [];
  const reconnectedEdges = reconnectAroundDisabledNodes(edges, enabledNodeIds);

  const convertedEdges: EngineReadyEdge[] = reconnectedEdges.map((edge) => {
    return {
      id: edge.id,
      from: edge.source,
      to: edge.target,
      fromPort: edge.sourceHandle ?? DEFAULT_EDGE_PORT,
      toPort: edge.targetHandle ?? DEFAULT_EDGE_PORT,
    };
  });
  return convertedEdges;
};
