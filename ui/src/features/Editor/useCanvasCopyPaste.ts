import { addEdge } from "@xyflow/react";
import { useCallback } from "react";

import { useCopyPaste } from "@flow/hooks/useCopyPaste";
import { Edge, Node } from "@flow/types";
import { randomID } from "@flow/utils";

export default ({
  nodes,
  edges,
  handleNodesUpdate,
  handleEdgesUpdate,
}: {
  nodes: Node[];
  edges: Edge[];
  handleNodesUpdate: (newNodes: Node[]) => void;
  handleEdgesUpdate: (newEdges: Edge[]) => void;
}) => {
  const { copy, paste } = useCopyPaste<
    { nodeIds: string[]; edges: Edge[] } | undefined
  >();

  const handleCopy = useCallback(() => {
    const selected: { nodeIds: string[]; edges: Edge[] } | undefined = {
      nodeIds: nodes.filter((n) => n.selected).map((n) => n.id),
      edges: edges.filter((e) => e.selected),
    };
    if (selected.nodeIds.length === 0 && selected.edges.length === 0) return;
    copy(selected);
  }, [nodes, edges, copy]);

  const handlePaste = useCallback(() => {
    const { nodeIds: pnid, edges: pe } = paste() || { nodeIds: [], edges: [] };

    const pn = nodes.filter((n) => pnid.includes(n.id));

    const idMapArray: { prevId: string; newId: string }[] = [];

    const newNodes: Node[] = [];
    for (const n of pn) {
      // if NOT a child of a batch, offset position for user's benefit
      const newPosition = n.parentId
        ? { x: n.position.x, y: n.position.y }
        : { x: n.position.x + 40, y: n.position.y + 20 };
      const newNode: Node = {
        ...n,
        id: randomID(),
        position: newPosition,
        selected: true, // select pasted nodes
      };

      if (newNode.type === "batch") {
        idMapArray.push({ prevId: n.id, newId: newNode.id });
      }

      newNodes.push(newNode);
    }

    // Update parentIds for nodes that are batched
    const reBatchedNodes: Node[] = newNodes.map((nn) => {
      const rbn = nn;
      const newParentId = idMapArray.find(
        (idMap) => idMap.prevId === nn.parentId,
      )?.newId;
      if (newParentId) {
        return { ...rbn, parentId: newParentId };
      }
      return rbn;
    });

    let newEdges: Edge[] = edges;
    for (const e of pe) {
      const sourceNode =
        reBatchedNodes[pn?.findIndex((n) => n.id === e.source)];
      const targetNode =
        reBatchedNodes[pn?.findIndex((n) => n.id === e.target)];

      if (!sourceNode || !targetNode) continue;

      newEdges = addEdge(
        {
          source: sourceNode.id,
          target: targetNode.id,
          sourceHandle: e.sourceHandle ?? null,
          targetHandle: e.targetHandle ?? null,
        },
        newEdges,
      );
    }

    // Copy new nodes and edges. Since they are selected now,
    // if the user pastes again, the new nodes and edges will
    // be what is pasted with an appropriate offset position.
    copy({
      nodeIds: reBatchedNodes.map((n) => n.id),
      edges: newEdges.filter((e) => !edges.find((e2) => e2.id === e.id)),
    });

    handleNodesUpdate([
      ...nodes.map((n) => ({ ...n, selected: false })), // deselect all previously selected nodes
      ...reBatchedNodes,
    ]);

    handleEdgesUpdate(newEdges);
  }, [nodes, edges, copy, paste, handleNodesUpdate, handleEdgesUpdate]);

  return {
    handleCopy,
    handlePaste,
  };
};
