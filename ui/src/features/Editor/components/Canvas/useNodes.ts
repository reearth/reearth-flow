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
import { Dispatch, MouseEvent, SetStateAction } from "react";

import type { Edge, Node } from "@flow/types";

import useBatch from "./useBatch";
import useDnd from "./useDnd";

type Props = {
  nodes: Node[];
  edges: Edge[];
  setNodes: Dispatch<SetStateAction<Node[]>>;
  setEdges: Dispatch<SetStateAction<Edge[]>>;
};

export default ({ nodes, edges, setNodes, setEdges }: Props) => {
  const { isNodeIntersecting } = useReactFlow();
  const { handleNodeDropInBatch } = useBatch();
  const { handleNodeDragOver, handleNodeDrop } = useDnd({ setNodes });

  const handleNodesChange: OnNodesChange<Node> = changes => {
    setNodes(nds => applyNodeChanges<Node>(changes, nds));
  };

  const handleNodesDelete = (deleted: Node[]) => {
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
  };

  const handleNodeDropOnEdge = (node: Node) => {
    edges.forEach(e => {
      const sourceNode = nodes.find(n => n.id === e.source);
      const targetNode = nodes.find(n => n.id === e.target);

      if (!sourceNode || !targetNode) return;

      const [, labelX, labelY] = getBezierPath({
        sourceX: sourceNode.position.x,
        sourceY: sourceNode.position.y,
        sourcePosition: sourceNode.sourcePosition,
        targetX: targetNode.position.x,
        targetY: targetNode.position.y,
        targetPosition: targetNode.targetPosition,
      });
      if (
        isNodeIntersecting(node, { x: labelX - 30, y: labelY - 30, width: 60, height: 60 }, true)
      ) {
        // remove edge
        const newEdges = edges.filter(ed => ed.id !== e.id);
        // create new connection between original source node and dragged node
        const newConnectionA: Connection = {
          source: e.source,
          sourceHandle: e.sourceHandle ?? null,
          target: node.id,
          targetHandle: node.handles?.find(h => h.type === "target")?.type ?? null,
        };
        const newEdgesA = addEdge(newConnectionA, newEdges);

        // create new connection between dragged node and original target node
        const newConnectionB: Connection = {
          source: node.id,
          sourceHandle: node.handles?.find(h => h.type === "source")?.type ?? null,
          target: e.target,
          targetHandle: e.targetHandle ?? null,
        };
        const newEdgesB = addEdge(newConnectionB, newEdgesA);

        // update nodes
        setEdges(newEdgesB);
      }
    });
  };

  const handleNodeDragStop = (_evt: MouseEvent, node: Node) => {
    if (node.type !== "batch") {
      handleNodeDropInBatch(node, nodes, setNodes);
      if (node.type !== "note") {
        handleNodeDropOnEdge(node);
      }
    }
  };

  return {
    handleNodesChange,
    handleNodesDelete,
    handleNodeDragStop,
    handleNodeDrop,
    handleNodeDragOver,
  };
};
