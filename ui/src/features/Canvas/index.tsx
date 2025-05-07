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
  isValidConnection,
  CustomConnectionLine,
  edgeTypes,
  connectionLineStyle,
  nodeTypes,
} from "@flow/lib/reactFlow";
import type { ActionNodeType, Edge, Node } from "@flow/types";

import {
  NodeContextMenu,
  NodeDeletionDialog,
  PaneContextMenu,
  SelectionContextMenu,
} from "./components";
import useHooks, { defaultEdgeOptions } from "./hooks";

import "@xyflow/react/dist/style.css";

const gridSize = 25;

const snapGrid: SnapGrid = [gridSize, gridSize];

const proOptions: ProOptions = { hideAttribution: true };

type Props = {
  isSubworkflow: boolean;
  nodes: Node[];
  edges: Edge[];
  selectedEdgeIds?: string[];
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
  onCopy?: () => void;
  onCut?: () => void;
  onPaste?: () => void;
};

const Canvas: React.FC<Props> = ({
  isSubworkflow,
  canvasLock,
  nodes,
  edges,
  selectedEdgeIds,
  onWorkflowAdd,
  onNodesAdd,
  onNodesChange,
  onNodeDoubleClick,
  onNodeHover,
  onEdgeHover,
  onEdgesAdd,
  onEdgesChange,
  onNodePickerOpen,
  onCopy,
  onCut,
  onPaste,
}) => {
  const {
    handleNodesChange,
    handleNodesDelete,
    handleBeforeDeleteNodes,
    handleNodeDragStop,
    handleNodeDragOver,
    handleNodeDrop,
    handleNodeDoubleClick,
    handleEdgesChange,
    handleConnect,
    handleReconnect,
    handleNodeContextMenu,
    handleSelectionContextMenu,
    handlePaneContextMenu,
    handleCloseContextmenu,
    contextMenu,
    paneRef,
    showBeforeDeleteDialog,
    handleDeleteDialogClose,
    deferredDeleteRef,
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
    <ReactFlow
      className={`${isSubworkflow ? "border-node-subworkflow border" : ""}`}
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
      ref={paneRef}
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
      onNodeDragStart={handleCloseContextmenu}
      onNodeDragStop={handleNodeDragStop}
      onNodesDelete={handleNodesDelete}
      onNodeMouseEnter={onNodeHover}
      onNodeMouseLeave={onNodeHover}
      onNodeContextMenu={handleNodeContextMenu}
      onSelectionContextMenu={handleSelectionContextMenu}
      onPaneContextMenu={handlePaneContextMenu}
      onMoveStart={handleCloseContextmenu}
      onDrop={handleNodeDrop}
      onDragOver={handleNodeDragOver}
      onEdgeMouseEnter={onEdgeHover}
      onEdgeMouseLeave={onEdgeHover}
      onConnect={handleConnect}
      onReconnect={handleReconnect}
      onBeforeDelete={handleBeforeDeleteNodes}
      proOptions={proOptions}>
      <Background
        className="bg-background"
        variant={BackgroundVariant["Lines"]}
        gap={gridSize}
        color="rgba(63, 63, 70, 0.4)"
      />

      {contextMenu?.type === "node" && (
        <NodeContextMenu
          node={contextMenu.data}
          contextMenu={contextMenu}
          onCopy={onCopy}
          onCut={onCut}
          onNodesChange={handleNodesChange}
          onSecondaryNodeAction={onNodeDoubleClick}
          onClose={handleCloseContextmenu}
        />
      )}
      {contextMenu?.type === "selection" && (
        <SelectionContextMenu
          nodes={contextMenu.data}
          selectedEdgeIds={selectedEdgeIds}
          contextMenu={contextMenu}
          onNodesChange={handleNodesChange}
          onEdgesChange={handleEdgesChange}
          onCopy={onCopy}
          onCut={onCut}
          onPaste={onPaste}
          onClose={handleCloseContextmenu}
        />
      )}
      {contextMenu?.type === "pane" && (
        <PaneContextMenu
          contextMenu={contextMenu}
          onCopy={onCopy}
          onPaste={onPaste}
          onClose={handleCloseContextmenu}
        />
      )}
      {showBeforeDeleteDialog && (
        <NodeDeletionDialog
          showBeforeDeleteDialog={showBeforeDeleteDialog}
          deferredDeleteRef={deferredDeleteRef}
          onDialogClose={handleDeleteDialogClose}
        />
      )}
    </ReactFlow>
  );
};

export default memo(Canvas);
