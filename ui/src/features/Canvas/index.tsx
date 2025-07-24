import {
  ReactFlow,
  Background,
  BackgroundVariant,
  SelectionMode,
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

import { CanvasContextMenu } from "./components";
import useHooks, { defaultEdgeOptions } from "./hooks";

import "@xyflow/react/dist/style.css";

const gridSize = 16.5;

const snapGrid: SnapGrid = [gridSize, gridSize];

type Props = {
  readonly?: boolean;
  nodes: Node[];
  edges: Edge[];
  selectedEdgeIds?: string[];
  onWorkflowAdd?: (position?: XYPosition) => void;
  onWorkflowOpen?: (workflowId: string) => void;
  onNodesAdd?: (newNode: Node[]) => void;
  onNodesChange?: (changes: NodeChange<Node>[]) => void;
  onBeforeDelete?: (args: { nodes: Node[] }) => Promise<boolean>;
  onNodeSettings?: (e: MouseEvent | undefined, nodeId: string) => void;
  onNodeHover?: (e: MouseEvent, node?: Node) => void;
  onNodePickerOpen?: (position: XYPosition, nodeType?: ActionNodeType) => void;
  onEdgesAdd?: (newEdges: Edge[]) => void;
  onEdgesChange?: (changes: EdgeChange[]) => void;
  onEdgeHover?: (e: MouseEvent, edge?: Edge) => void;
  onCopy?: (node?: Node) => void;
  onCut?: (isCutByShortCut?: boolean, node?: Node) => void;
  onPaste?: () => void;
};

const Canvas: React.FC<Props> = ({
  readonly,
  nodes,
  edges,
  selectedEdgeIds,
  onWorkflowAdd,
  onWorkflowOpen,
  onNodesAdd,
  onNodesChange,
  onBeforeDelete,
  onNodeSettings,
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
    handleNodesDelete,
    handleNodeDragStop,
    handleNodeDragOver,
    handleNodeDrop,
    handleNodeSettings,
    handleConnect,
    handleReconnect,
    handleNodeContextMenu,
    handleSelectionContextMenu,
    handlePaneContextMenu,
    handleCloseContextmenu,
    contextMenu,
    paneRef,
  } = useHooks({
    nodes,
    edges,
    onWorkflowAdd,
    onNodesAdd,
    onNodesChange,
    onNodeSettings,
    onEdgesAdd,
    onEdgesChange,
    onNodePickerOpen,
  });

  return (
    <ReactFlow
      ref={paneRef}
      // Readonly props START
      nodesConnectable={!readonly}
      nodesFocusable={!readonly}
      elementsSelectable={!readonly}
      reconnectRadius={!readonly ? 10 : 0}
      // Readonly props END
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
      onNodesChange={onNodesChange}
      onEdgesChange={onEdgesChange}
      onNodeDoubleClick={handleNodeSettings}
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
      onBeforeDelete={onBeforeDelete}>
      <Background
        className="bg-background"
        variant={BackgroundVariant["Dots"]}
        gap={gridSize}
        color="rgba(63, 63, 70, 1)"
      />
      {contextMenu && (
        <CanvasContextMenu
          data={contextMenu.data}
          selectedEdgeIds={selectedEdgeIds}
          contextMenu={contextMenu}
          onBeforeDelete={onBeforeDelete}
          onNodesChange={onNodesChange}
          onEdgesChange={onEdgesChange}
          onWorkflowOpen={onWorkflowOpen}
          onNodeSettings={onNodeSettings}
          onCopy={onCopy}
          onCut={onCut}
          onPaste={onPaste}
          onClose={handleCloseContextmenu}
        />
      )}
    </ReactFlow>
  );
};

export default memo(Canvas);
