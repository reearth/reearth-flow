import {
  Connection,
  OnNodesChange,
  XYPosition,
  addEdge,
  applyNodeChanges,
  getBezierPath,
  getConnectedEdges,
  getIncomers,
  getOutgoers,
  useReactFlow,
} from "@xyflow/react";
import { MouseEvent, useCallback } from "react";

import type { ActionNodeType, Edge, Node } from "@flow/types";

import useBatch from "./useBatch";
import useDnd from "./useDnd";

type Props = {
  nodes: Node[];
  edges: Edge[];
  onWorkflowAdd: (position?: XYPosition) => void;
  onNodesChange: (newNodes: Node[]) => void;
  onEdgesChange: (edges: Edge[]) => void;
  onNodePickerOpen: (position: XYPosition, nodeType?: ActionNodeType) => void;
};

export default ({
  nodes,
  edges,
  onWorkflowAdd,
  onNodesChange,
  onEdgesChange,
  onNodePickerOpen,
}: Props) => {
  const { isNodeIntersecting } = useReactFlow();
  const { handleNodeDropInBatch } = useBatch();

  const { handleNodeDragOver, handleNodeDrop } = useDnd({
    nodes,
    onWorkflowAdd,
    onNodesChange,
    onNodePickerOpen,
    handleNodeDropInBatch,
  });

  const handleNodesChange: OnNodesChange<Node> = useCallback(
    (changes) => onNodesChange(applyNodeChanges<Node>(changes, nodes)),
    [nodes, onNodesChange],
  );

  const handleNodesDelete = useCallback(
    (deleted: Node[]) => {
      // If a deleted node is connected between two remaining nodes,
      // on removal, create a new connection between those nodes
      onEdgesChange(
        deleted.reduce((acc, node) => {
          const incomers = getIncomers(node, nodes, edges);
          const outgoers = getOutgoers(node, nodes, edges);
          const connectedEdges = getConnectedEdges([node], edges);

          const remainingEdges = acc.filter(
            (edge) => !connectedEdges.includes(edge),
          );

          const createdEdges = incomers.flatMap(({ id: source }) =>
            outgoers.map(({ id: target }) => ({
              id: `${source}->${target}`,
              source,
              target,
            })),
          );

          return [...remainingEdges, ...createdEdges];
        }, edges),
      );
    },
    [edges, nodes, onEdgesChange],
  );

  const handleNodeDropOnEdge = useCallback(
    (droppedNode: Node) => {
      if (!droppedNode.data.inputs || !droppedNode.data.outputs) return;

      let edgeCreationComplete = false;

      // Make sure dropped node is empty
      const connectedEdges = edges.filter(
        (e) => e.source === droppedNode.id || e.target === droppedNode.id,
      );
      if (connectedEdges && connectedEdges.length > 0) return;

      for (const edge of edges) {
        // Stop loop if an edge was created already after node drop
        if (edgeCreationComplete) break;

        const e = edge;

        // Make sure edge has source and target nodes
        const sourceNode = nodes.find((n) => n.id === e.source);
        const targetNode = nodes.find((n) => n.id === e.target);
        if (!sourceNode || !targetNode) return;

        let sourceNodeXYPosition: XYPosition = sourceNode.position;
        let targetNodeXYPosition: XYPosition = targetNode.position;

        // If source or target node is inside a group, calculate its position relative to the group
        if (sourceNode.parentId) {
          const parentNode = nodes.find((n) => n.id === sourceNode.parentId);
          if (parentNode) {
            sourceNodeXYPosition = {
              x: parentNode.position.x + sourceNode.position.x,
              y: parentNode.position.y + sourceNode.position.y,
            };
          }
        }
        if (targetNode.parentId) {
          const parentNode = nodes.find((n) => n.id === targetNode.parentId);
          if (parentNode) {
            targetNodeXYPosition = {
              x: parentNode.position.x + targetNode.position.x,
              y: parentNode.position.y + targetNode.position.y,
            };
          }
        }

        // Get middle of edge
        const [, labelX, labelY] = getBezierPath({
          sourceX: sourceNodeXYPosition.x,
          sourceY: sourceNodeXYPosition.y,
          sourcePosition: sourceNode.sourcePosition,
          targetX: targetNodeXYPosition.x,
          targetY: targetNodeXYPosition.y,
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
          let newEdges = edges.filter((ed) => ed.id !== e.id);
          // create new connection between original source node and dragged node
          const newConnectionA: Connection = {
            source: e.source,
            sourceHandle: e.sourceHandle ?? null,
            target: droppedNode.id,
            targetHandle:
              droppedNode.handles?.find((h) => h.type === "target")?.type ??
              null,
          };
          newEdges = addEdge(newConnectionA, newEdges);

          // create new connection between dragged node and original target node
          const newConnectionB: Connection = {
            source: droppedNode.id,
            sourceHandle:
              droppedNode.handles?.find((h) => h.type === "source")?.type ??
              null,
            target: e.target,
            targetHandle: e.targetHandle ?? null,
          };
          newEdges = addEdge(newConnectionB, newEdges);

          onEdgesChange(newEdges);

          // Set edge creation complete to stop loop
          edgeCreationComplete = true;
        }
      }
    },
    [edges, isNodeIntersecting, nodes, onEdgesChange],
  );

  const handleNodeDragStop = useCallback(
    (_evt: MouseEvent, node: Node) => {
      if (node.type !== "batch") {
        onNodesChange(handleNodeDropInBatch(node, nodes));
        if (node.type !== "note") {
          handleNodeDropOnEdge(node);
        }
      }
    },
    [handleNodeDropInBatch, handleNodeDropOnEdge, nodes, onNodesChange],
  );

  return {
    handleNodesChange,
    handleNodesDelete,
    handleNodeDragStop,
    handleNodeDrop,
    handleNodeDragOver,
  };
};
