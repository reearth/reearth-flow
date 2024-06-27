import {
  Connection,
  OnNodesChange,
  addEdge,
  applyNodeChanges,
  getBezierPath,
  getConnectedEdges,
  getIncomers,
  getOutgoers,
  useReactFlow,
} from "@xyflow/react";
import { Dispatch, MouseEvent, SetStateAction, useCallback } from "react";

import type { Edge, Node } from "@flow/types";

import useBatch from "./useBatch";
import useDnd from "./useDnd";

type Props = {
  nodes: Node[];
  edges: Edge[];
  setNodes: Dispatch<SetStateAction<Node[]>>;
  setEdges: Dispatch<SetStateAction<Edge[]>>;
  onNodeLocking: (nodeId: string, setNodes: Dispatch<SetStateAction<Node[]>>) => void;
};

export default ({ nodes, edges, setNodes, setEdges, onNodeLocking }: Props) => {
  const { isNodeIntersecting } = useReactFlow();
  const { handleNodeDropInBatch } = useBatch();
  const { handleNodeDragOver, handleNodeDrop } = useDnd({ setNodes, onNodeLocking });

  const handleNodesChange: OnNodesChange<Node> = useCallback(
    changes => {
      setNodes(nds => applyNodeChanges<Node>(changes, nds));
    },
    [setNodes],
  );

  const handleNodesDelete = useCallback(
    (deleted: Node[]) => {
      // If a deleted node is connected between two remaining nodes,
      // on removal, create a new connection between those nodes
      setEdges(
        deleted.reduce((acc, node) => {
          const incomers = getIncomers(node, nodes, edges);
          const outgoers = getOutgoers(node, nodes, edges);
          const connectedEdges = getConnectedEdges([node], edges);

          const remainingEdges = acc.filter(edge => !connectedEdges.includes(edge));

          const createdEdges = incomers.flatMap(({ id: source }) =>
            outgoers.map(({ id: target }) => ({ id: `${source}->${target}`, source, target })),
          );

          return [...remainingEdges, ...createdEdges];
        }, edges),
      );
    },
    [edges, nodes, setEdges],
  );

  const handleNodeDropOnEdge = useCallback(
    (droppedNode: Node) => {
      if (!droppedNode.data.inputs || !droppedNode.data.outputs) return;

      let edgeCreationComplete = false;

      // Make sure dropped node is empty
      const connectedEdges = edges.filter(
        e => e.source === droppedNode.id || e.target === droppedNode.id,
      );
      if (connectedEdges && connectedEdges.length > 0) return;

      for (let i = 0; i < edges.length; i++) {
        // Stop loop if an edge was created already after node drop
        if (edgeCreationComplete) break;

        const e = edges[i];

        // Make sure edge has source and target nodes
        const sourceNode = nodes.find(n => n.id === e.source);
        const targetNode = nodes.find(n => n.id === e.target);
        if (!sourceNode || !targetNode) return;

        // Get middle of edge
        const [, labelX, labelY] = getBezierPath({
          sourceX: sourceNode.position.x,
          sourceY: sourceNode.position.y,
          sourcePosition: sourceNode.sourcePosition,
          targetX: targetNode.position.x,
          targetY: targetNode.position.y,
          targetPosition: targetNode.targetPosition,
        });

        // Check if dropped node is intersecting with edge's middle
        if (
          isNodeIntersecting(
            droppedNode,
            { x: labelX - 30, y: labelY - 30, width: 60, height: 60 },
            true,
          )
        ) {
          // remove previous edge
          let newEdges = edges.filter(ed => ed.id !== e.id);
          // create new connection between original source node and dragged node
          const newConnectionA: Connection = {
            source: e.source,
            sourceHandle: e.sourceHandle ?? null,
            target: droppedNode.id,
            targetHandle: droppedNode.handles?.find(h => h.type === "target")?.type ?? null,
          };
          newEdges = addEdge(newConnectionA, newEdges);

          // create new connection between dragged node and original target node
          const newConnectionB: Connection = {
            source: droppedNode.id,
            sourceHandle: droppedNode.handles?.find(h => h.type === "source")?.type ?? null,
            target: e.target,
            targetHandle: e.targetHandle ?? null,
          };
          newEdges = addEdge(newConnectionB, newEdges);

          setEdges(newEdges);

          // Set edge creation complete to stop loop
          edgeCreationComplete = true;
        }
      }
    },
    [edges, isNodeIntersecting, nodes, setEdges],
  );

  const handleNodeDragStop = useCallback(
    (_evt: MouseEvent, node: Node) => {
      if (node.type !== "batch") {
        handleNodeDropInBatch(node, nodes, setNodes);
        if (node.type !== "note") {
          handleNodeDropOnEdge(node);
        }
      }
    },
    [handleNodeDropInBatch, handleNodeDropOnEdge, nodes, setNodes],
  );

  return {
    handleNodesChange,
    handleNodesDelete,
    handleNodeDragStop,
    handleNodeDrop,
    handleNodeDragOver,
  };
};
