import { Database, Disc, Lightning } from "@phosphor-icons/react";
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
import { MouseEvent, memo } from "react";

import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuTrigger,
} from "@flow/components";
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

const gridSize = 30;

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
  return (
    <ContextMenu>
      <ContextMenuTrigger asChild>
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
          proOptions={proOptions}>
          <Background
            className="bg-background"
            variant={BackgroundVariant["Lines"]}
            gap={gridSize}
            color="rgba(63, 63, 70, 0.3)"
          />
        </ReactFlow>
      </ContextMenuTrigger>
      <ContextMenuContent>
        {!nodes.some((node) => node.type === "reader") && (
          <ContextMenuItem
            className="justify-between gap-4 text-xs"
            onClick={(e) =>
              onNodePickerOpen &&
              onNodePickerOpen({ x: e.clientX, y: e.clientY }, "reader")
            }>
            <Database weight="light" /> Add Reader
          </ContextMenuItem>
        )}
        <ContextMenuItem
          className="justify-between gap-4 text-xs"
          onClick={(e) =>
            onNodePickerOpen &&
            onNodePickerOpen({ x: e.clientX, y: e.clientY }, "transformer")
          }>
          <Lightning weight="light" /> Add Transformer
        </ContextMenuItem>
        <ContextMenuItem
          className="justify-between gap-4 text-xs"
          onClick={(e) =>
            onNodePickerOpen &&
            onNodePickerOpen({ x: e.clientX, y: e.clientY }, "writer")
          }>
          <Disc weight="light" />
          Add Writer
        </ContextMenuItem>
      </ContextMenuContent>
    </ContextMenu>
  );
};

export default memo(Canvas);
