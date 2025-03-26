import {
  ReactFlow,
  Background,
  BackgroundVariant,
  SelectionMode,
  ProOptions,
  SnapGrid,
  XYPosition,
  NodeChange,
  EdgeChange,
} from "@xyflow/react";
import { MouseEvent, memo, useCallback, useRef, useState } from "react";

import {
  isValidConnection,
  CustomConnectionLine,
  edgeTypes,
  connectionLineStyle,
  nodeTypes,
} from "@flow/lib/reactFlow";
import type { ActionNodeType, Edge, Node } from "@flow/types";

import useHooks, { defaultEdgeOptions } from "./hooks";

import "@xyflow/react/dist/style.css";
import { NodeContextMenu } from "./components";
import { MenuPosition } from "@flow/components";

const gridSize = 25;

const snapGrid: SnapGrid = [gridSize, gridSize];

const proOptions: ProOptions = { hideAttribution: true };

type Props = {
  nodes: Node[];
  edges: Edge[];
  canvasLock: boolean;
  onWorkflowAdd?: (position?: XYPosition) => void;
  onNodesAdd?: (newNode: Node[]) => void;
  onNodesChange?: (changes: NodeChange<Node>[]) => void;
  onNodeDoubleClick?: (
    e: MouseEvent | undefined,
    nodeId: string,
    subworkflowId?: string,
  ) => void;
  onNodeHover?: (e: MouseEvent, node?: Node) => void;
  onNodePickerOpen?: (position: XYPosition, nodeType?: ActionNodeType) => void;
  onEdgesAdd?: (newEdges: Edge[]) => void;
  onEdgesChange?: (changes: EdgeChange[]) => void;
  onEdgeHover?: (e: MouseEvent, edge?: Edge) => void;
};

const Canvas: React.FC<Props> = ({
  canvasLock,
  nodes,
  edges,
  onWorkflowAdd,
  onNodesAdd,
  onNodesChange,
  onNodeDoubleClick,
  onNodeHover,
  onEdgeHover,
  onEdgesAdd,
  onEdgesChange,
  onNodePickerOpen,
}) => {
  const {
    handleNodesChange,
    handleNodesDelete,
    handleNodeDragStop,
    handleNodeDragOver,
    handleNodeDrop,
    handleNodeDoubleClick,
    handleEdgesChange,
    handleConnect,
    handleReconnect,
  } = useHooks({
    nodes,
    edges,
    onWorkflowAdd,
    onNodesAdd,
    onNodesChange,
    onNodeDoubleClick,
    onEdgesAdd,
    onEdgesChange,
    onNodePickerOpen,
  });

  const [menuPosition, setMenuPosition] = useState<MenuPosition | null>(null);

  const [selectedNode, setSelectedNode] = useState<Node | null>(null);
  const ref = useRef<HTMLDivElement>(null);

  const onNodeContextMenu = useCallback(
    (event: MouseEvent, node: Node) => {
      event.preventDefault();
      if (!ref.current) return;
      const pane = ref.current.getBoundingClientRect();
      setMenuPosition({
        top: event.clientY < pane.height - 200 && event.clientY,
        left: event.clientX < pane.width - 200 && event.clientX,
        right: event.clientX >= pane.width - 200 && pane.width - event.clientX,
        bottom:
          event.clientY >= pane.height - 200 && pane.height - event.clientY,
      });

      setSelectedNode(node);
    },
    [setMenuPosition],
  );

  const closeMenu = () => {
    setMenuPosition(null);
  };

  return (
    <ReactFlow
      // minZoom={0.7}
      // maxZoom={1}
      // defaultViewport={{ zoom: 0.8, x: 200, y: 200 }}
      // translateExtent={[
      //   [-1000, -1000],
      //   [1000, 1000],
      // ]}
      // onInit={setReactFlowInstance}
      // selectNodesOnDrag={false}
      // fitViewOptions={{ padding: 0.5 }}
      // fitView
      ref={ref}
      // Locking props START
      nodesDraggable={!canvasLock}
      nodesConnectable={!canvasLock}
      nodesFocusable={!canvasLock}
      edgesFocusable={!canvasLock}
      // elementsSelectable={!canvasLock}
      autoPanOnConnect={!canvasLock}
      autoPanOnNodeDrag={!canvasLock}
      // panOnDrag={!canvasLock}
      selectionOnDrag={!canvasLock}
      // panOnScroll={!canvasLock}
      // zoomOnScroll={!canvasLock}
      // zoomOnPinch={!canvasLock}
      // zoomOnDoubleClick={!canvasLock}
      connectOnClick={!canvasLock}
      // Locking props END

      nodeDragThreshold={2}
      snapToGrid
      snapGrid={snapGrid}
      selectionMode={SelectionMode["Partial"]}
      nodes={nodes}
      nodeTypes={nodeTypes}
      edges={edges}
      edgeTypes={edgeTypes}
      defaultEdgeOptions={defaultEdgeOptions}
      connectionLineComponent={CustomConnectionLine}
      connectionLineStyle={connectionLineStyle}
      isValidConnection={isValidConnection}
      onNodesChange={handleNodesChange}
      onEdgesChange={handleEdgesChange}
      onNodeDoubleClick={handleNodeDoubleClick}
      onNodeDragStop={handleNodeDragStop}
      onNodesDelete={handleNodesDelete}
      onNodeMouseEnter={onNodeHover}
      onNodeMouseLeave={onNodeHover}
      onDrop={handleNodeDrop}
      onDragOver={handleNodeDragOver}
      onEdgeMouseEnter={onEdgeHover}
      onEdgeMouseLeave={onEdgeHover}
      onConnect={handleConnect}
      onReconnect={handleReconnect}
      proOptions={proOptions}
      onPaneClick={closeMenu}
      onNodeContextMenu={onNodeContextMenu}>
      <Background
        className="bg-background"
        variant={BackgroundVariant["Lines"]}
        gap={gridSize}
        color="rgba(63, 63, 70, 0.3)"
      />

      {menuPosition && selectedNode && (
        <NodeContextMenu
          node={selectedNode}
          menuPosition={menuPosition}
          onClose={closeMenu}
        />
      )}
    </ReactFlow>
  );
};

export default memo(Canvas);
