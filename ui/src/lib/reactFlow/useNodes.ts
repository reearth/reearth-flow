import {
  EdgeChange,
  NodeChange,
  OnNodesChange,
  XYPosition,
  getBezierPath,
  getConnectedEdges,
  getIncomers,
  getOutgoers,
  useReactFlow,
} from "@xyflow/react";
import { MouseEvent, useCallback } from "react";

import type { ActionNodeType, Edge, Node } from "@flow/types";
import { generateUUID } from "@flow/utils";

import useBatch from "./useBatch";
import useDnd from "./useDnd";

type Props = {
  nodes: Node[];
  edges: Edge[];
  onWorkflowAdd?: (position?: XYPosition) => void;
  onNodesAdd?: (newNode: Node[]) => void;
  onNodesChange?: (changes: NodeChange<Node>[]) => void;
  onNodeSettings?: (e: MouseEvent | undefined, nodeId: string) => void;
  onEdgesChange?: (changes: EdgeChange[]) => void;
  onNodePickerOpen?: (position: XYPosition, nodeType?: ActionNodeType) => void;
};

export default ({
  nodes,
  edges,
  onWorkflowAdd,
  onNodesAdd,
  onNodesChange,
  onNodeSettings,
  onEdgesChange,
  onNodePickerOpen,
}: Props) => {
  const { isNodeIntersecting } = useReactFlow();

  const { handleNodeDragOver, handleNodeDrop } = useDnd({
    onWorkflowAdd,
    onNodesAdd,
    onNodePickerOpen,
  });

  const { handleNodesDropInBatch } = useBatch();

  const handleNodesChange: OnNodesChange<Node> = useCallback(
    (changes) => onNodesChange?.(changes),
    [onNodesChange],
  );

  const handleNodesDeleteCleanup = useCallback(
    (deleted: Node[]) => {
      // We use deletedIds below to make sure we don't create new connections between nodes
      // that are being deleted
      const deletedIds = new Set(deleted.map((node) => node.id));

      const changes: EdgeChange[] = deleted.reduce((acc, node) => {
        const incomers = getIncomers(node, nodes, edges);
        const outgoers = getOutgoers(node, nodes, edges);
        const connectedEdges = getConnectedEdges([node], edges);

        // First, mark all connected edges for removal
        const removals: EdgeChange[] = connectedEdges.map((edge) => ({
          id: edge.id,
          type: "remove" as const,
        }));

        // If node has multiple inputs or outputs, we don't want to create new connections
        // due to the ambiguity of which input/output to connect to
        const hasMultiple =
          (node.data.outputs && node.data.outputs.length > 1) ||
          (node.data.inputs && node.data.inputs.length > 1);

        // Then create new direct connections between incomers and outgoers
        const additions: EdgeChange[] = !hasMultiple
          ? incomers
              .filter(({ id }) => !deletedIds.has(id))
              .flatMap(({ id: source }) =>
                outgoers
                  .filter(({ id }) => !deletedIds.has(id))
                  .map(({ id: target }) => ({
                    id: `${source}->${target}`,
                    type: "add" as const,
                    item: {
                      id: `${source}->${target}`,
                      source,
                      target,
                    },
                  })),
              )
          : [];

        return [...acc, ...removals, ...additions];
      }, [] as EdgeChange[]);

      onEdgesChange?.(changes);
    },
    [edges, nodes, onEdgesChange],
  );

  const handleNodeDropOnEdge = useCallback(
    (droppedNode: Node) => {
      if (
        droppedNode.type === "subworkflow" &&
        (!droppedNode.data.pseudoOutputs?.length ||
          !droppedNode.data.pseudoInputs?.length)
      ) {
        return;
      } else if (
        droppedNode.type !== "subworkflow" &&
        (!droppedNode.data.outputs?.length || !droppedNode.data.inputs?.length)
      ) {
        return;
      }

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
          const removeChanges: EdgeChange[] = [
            {
              id: edge.id,
              type: "remove" as const,
            },
          ];
          const addChanges: EdgeChange[] = [
            {
              type: "add" as const,
              item: {
                id: generateUUID(),
                source: e.source,
                target: droppedNode.id,
                sourceHandle: e.sourceHandle ?? null,
                targetHandle:
                  droppedNode.handles?.find((h) => h.type === "target")?.type ??
                  null,
              },
            },
            {
              type: "add" as const,
              item: {
                id: generateUUID(),
                source: droppedNode.id,
                target: e.target,
                sourceHandle:
                  droppedNode.handles?.find((h) => h.type === "source")?.type ??
                  null,
                targetHandle: e.targetHandle ?? null,
              },
            },
          ];

          onEdgesChange?.([...removeChanges, ...addChanges]);

          // Set edge creation complete to stop loop
          edgeCreationComplete = true;
        }
      }
    },
    [nodes, edges, isNodeIntersecting, onEdgesChange],
  );

  const handleDropInBatch = useCallback(
    (selectedNodes: Node[]) => {
      const updatedNodes = handleNodesDropInBatch(selectedNodes);

      if (updatedNodes) {
        const changes = updatedNodes.map((node) => ({
          type: "replace" as const,
          id: node.id,
          item: node,
        }));

        handleNodesChange(changes);
      }
    },
    [handleNodesDropInBatch, handleNodesChange],
  );

  const handleNodeDragStop = useCallback(
    (_evt: MouseEvent, node: Node, selectedNodes: Node[]) => {
      if (node.type !== "batch") {
        handleDropInBatch(selectedNodes);
        if (node.type !== "note") {
          handleNodeDropOnEdge(node);
        }
      }
    },
    [handleNodeDropOnEdge, handleDropInBatch],
  );

  const handleNodeSettings = useCallback(
    (e: MouseEvent | undefined, node: Node) => {
      onNodeSettings?.(e, node.id);
    },
    [onNodeSettings],
  );

  return {
    handleNodesChange,
    handleNodesDeleteCleanup,
    handleNodeDragOver,
    handleNodeDragStop,
    handleNodeDrop,
    handleNodeSettings,
  };
};
