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

    const newNodes: Node[] = [];
    for (const n of pn) {
      const newNode: Node = {
        ...n,
        id: randomID(),
        position: { x: n.position.x + 40, y: n.position.y + 20 },
        selected: true, // select pasted nodes
      };
      newNodes.push(newNode);
    }

    let newEdges: Edge[] = edges;
    for (const e of pe) {
      const sourceNode = newNodes[pn?.findIndex((n) => n.id === e.source)];
      const targetNode = newNodes[pn?.findIndex((n) => n.id === e.target)];

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

    copy({
      nodeIds: newNodes.map((n) => n.id),
      edges: newEdges.filter((e) => !edges.find((e2) => e2.id === e.id)),
    });

    handleNodesUpdate([
      ...nodes.map((n) => ({ ...n, selected: false })), // deselect all previously selected nodes
      ...(newNodes || []),
    ]);

    handleEdgesUpdate(newEdges);
  }, [nodes, edges, copy, paste, handleNodesUpdate, handleEdgesUpdate]);

  return {
    handleCopy,
    handlePaste,
  };
};
