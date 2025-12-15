import { DEFAULT_EDGE_PORT } from "@flow/global-constants";
import type { Edge, EngineReadyEdge } from "@flow/types";

import { generateUUID } from "../generateUUID";

/**
 * Reconnects edges around disabled nodes to maintain workflow connectivity.
 * For example, if A → B → C and B is disabled, creates A → C.
 *
 * For nodes with multiple inputs/outputs, creates all possible connections (Cartesian product).
 * All reconnected edges use default ports for simplicity.
 *
 * This function iteratively processes disabled nodes until all are bypassed, which is necessary
 * for handling cases like A → B → C → D where both B and C are disabled (should result in A → D).
 */
const reconnectAroundDisabledNodes = (
  edges: Edge[],
  enabledNodeIds: Set<string>,
): Edge[] => {
  let currentEdges = edges;
  let hasChanges = true;

  // Keep processing until no more disabled nodes can be bypassed
  while (hasChanges) {
    hasChanges = false;

    // Find all disabled node IDs in current edge set
    const allNodeIds = new Set<string>();
    currentEdges.forEach(edge => {
      allNodeIds.add(edge.source);
      allNodeIds.add(edge.target);
    });

    const disabledNodeIds = Array.from(allNodeIds).filter(
      nodeId => !enabledNodeIds.has(nodeId)
    );

    if (disabledNodeIds.length === 0) {
      break;
    }

    // Build bypass connections for each disabled node
    const bypassEdges: Edge[] = [];

    disabledNodeIds.forEach(disabledNodeId => {
      // Find all edges coming into the disabled node
      const incomingEdges = currentEdges.filter(edge => edge.target === disabledNodeId);

      // Find all edges going out of the disabled node
      const outgoingEdges = currentEdges.filter(edge => edge.source === disabledNodeId);

      // If the disabled node has no incoming edges, it's a start node - skip it
      // If the disabled node has no outgoing edges, it's an end node - skip it
      if (incomingEdges.length === 0 || outgoingEdges.length === 0) {
        return;
      }

      // Create bypass edges: connect all inputs to all outputs (Cartesian product)
      incomingEdges.forEach(inEdge => {
        outgoingEdges.forEach(outEdge => {
          bypassEdges.push({
            id: generateUUID(),
            source: inEdge.source,
            target: outEdge.target,
            sourceHandle: DEFAULT_EDGE_PORT,
            targetHandle: DEFAULT_EDGE_PORT,
          });
        });
      });

      hasChanges = true;
    });

    // Filter out edges that connect to/from disabled nodes
    const validEdges = currentEdges.filter(
      edge => enabledNodeIds.has(edge.source) && enabledNodeIds.has(edge.target)
    );

    // Combine valid edges with bypass edges for next iteration
    currentEdges = [...validEdges, ...bypassEdges];
  }

  return currentEdges;
};

export const convertEdges = (enabledNodeIds: Set<string>, edges?: Edge[]) => {
  if (!edges) return [];

  // Reconnect edges around disabled nodes to maintain workflow connectivity
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
