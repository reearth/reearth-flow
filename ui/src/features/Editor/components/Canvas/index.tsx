import {
  ReactFlow,
  Background,
  BackgroundVariant,
  SelectionMode,
  ProOptions,
  SnapGrid,
} from "@xyflow/react";
import { MouseEvent, memo } from "react";

import type { Edge, Node } from "@flow/types";

import { CustomConnectionLine, edgeTypes, connectionLineStyle, nodeTypes } from "./components";
import useHooks, { defaultEdgeOptions } from "./hooks";

import "@xyflow/react/dist/style.css";

const gridSize = 30;

const snapGrid: SnapGrid = [gridSize, gridSize];

const proOptions: ProOptions = { hideAttribution: true };

type Props = {
  nodes: Node[];
  edges: Edge[];
  canvasLock: boolean;
  onNodesUpdate: (newNodes: Node[]) => void;
  onNodeLocking: (nodeId: string, nodes: Node[], onNodesChange: (nodes: Node[]) => void) => void;
  onNodeHover: (e: MouseEvent, node?: Node) => void;
  onEdgesUpdate: (newEdges: Edge[]) => void;
  onEdgeHover: (e: MouseEvent, edge?: Edge) => void;
};

const Canvas: React.FC<Props> = ({
  canvasLock,
  nodes,
  edges,
  onNodesUpdate,
  onNodeLocking,
  onNodeHover,
  onEdgeHover,
  onEdgesUpdate,
}) => {
  const {
    handleNodesChange,
    handleNodesDelete,
    handleNodeDragStop,
    handleNodeDragOver,
    handleNodeDrop,
    handleEdgesChange,
    handleConnect,
  } = useHooks({
    nodes,
    edges,
    onNodesUpdate,
    onEdgesUpdate,
    onNodeLocking,
  });

  return (
    <ReactFlow
      // minZoom={0.7}
      // maxZoom={1}
      // defaultViewport={{ zoom: 0.8, x: 200, y: 200 }}
      // nodeDragThreshold={60}
      // translateExtent={[
      //   [-1000, -1000],
      //   [1000, 1000],
      // ]}
      // onInit={setReactFlowInstance}
      // selectNodesOnDrag={false}
      // fitViewOptions={{ padding: 0.5 }}
      // fitView

      // Locking props START
      nodesDraggable={!canvasLock}
      nodesConnectable={!canvasLock}
      nodesFocusable={!canvasLock}
      edgesFocusable={!canvasLock}
      elementsSelectable={!canvasLock}
      autoPanOnConnect={!canvasLock}
      autoPanOnNodeDrag={!canvasLock}
      panOnDrag={!canvasLock}
      selectionOnDrag={!canvasLock}
      panOnScroll={!canvasLock}
      zoomOnScroll={!canvasLock}
      zoomOnPinch={!canvasLock}
      zoomOnDoubleClick={!canvasLock}
      connectOnClick={!canvasLock}
      // Locking props END

      snapGrid={snapGrid}
      selectionMode={SelectionMode["Partial"]}
      nodes={nodes}
      nodeTypes={nodeTypes}
      edges={edges}
      edgeTypes={edgeTypes}
      defaultEdgeOptions={defaultEdgeOptions}
      connectionLineComponent={CustomConnectionLine}
      connectionLineStyle={connectionLineStyle}
      onNodesChange={handleNodesChange}
      onEdgesChange={handleEdgesChange}
      onNodeDragStop={handleNodeDragStop}
      onNodesDelete={handleNodesDelete}
      onNodeMouseEnter={onNodeHover}
      onNodeMouseLeave={onNodeHover}
      onDrop={handleNodeDrop}
      onDragOver={handleNodeDragOver}
      onEdgeMouseEnter={onEdgeHover}
      onEdgeMouseLeave={onEdgeHover}
      onConnect={handleConnect}
      proOptions={proOptions}>
      <Background
        className="bg-zinc-800"
        variant={BackgroundVariant["Lines"]}
        gap={gridSize}
        color="rgba(63, 63, 70, 0.3)"
      />
    </ReactFlow>
  );
};

export default memo(Canvas);
