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
import type { Doc } from "yjs";

import {
  isValidConnection,
  CustomConnectionLine,
  fullEdgeTypes,
  simpleEdgeTypes,
  connectionLineStyle,
  nodeTypes,
} from "@flow/lib/reactFlow";
import type { ActionNodeType, AwarenessUser, Edge, Node } from "@flow/types";

import { CanvasContextMenu, MultiCursor } from "./components";
import useHooks, { defaultEdgeOptions } from "./hooks";

import "@xyflow/react/dist/style.css";

const gridSize = 16.5;

const snapGrid: SnapGrid = [gridSize, gridSize];

type Props = {
  readonly?: boolean;
  nodes: Node[];
  edges: Edge[];
  yDoc?: Doc | null;
  users?: Record<string, AwarenessUser>;
  currentWorkflowId?: string;
  isMainWorkflow: boolean;
  onWorkflowAdd?: (position?: XYPosition) => void;
  onWorkflowOpen?: (workflowId: string) => void;
  onWorkflowAddFromSelection?: (nodes: Node[], edges: Edge[]) => Promise<void>;
  onNodesAdd?: (newNode: Node[]) => void;
  onNodesChange?: (changes: NodeChange<Node>[]) => void;
  onBeforeDelete?: (args: { nodes: Node[] }) => Promise<boolean>;
  onNodeSettings?: (e: MouseEvent | undefined, nodeId: string) => void;
  onNodePickerOpen?: (
    position: XYPosition,
    nodeType?: ActionNodeType,
    isMainWorkflow?: boolean,
  ) => void;
  onNodesDisable?: (ns?: Node[] | undefined) => void;
  onEdgesAdd?: (newEdges: Edge[]) => void;
  onEdgesChange?: (changes: EdgeChange[]) => void;
  onCopy?: (node?: Node) => void;
  onCut?: (isCutByShortCut?: boolean, node?: Node) => void;
  onPaste?: () => void;
  onPaneMouseMove?: (e: MouseEvent) => void;
  onPaneClick?: (e: MouseEvent) => void;
};

const Canvas: React.FC<Props> = ({
  readonly,
  nodes,
  edges,
  users,
  currentWorkflowId,
  isMainWorkflow,
  onWorkflowAdd,
  onWorkflowOpen,
  onWorkflowAddFromSelection,
  onNodesAdd,
  onNodesChange,
  onBeforeDelete,
  onNodeSettings,
  onEdgesAdd,
  onEdgesChange,
  onNodePickerOpen,
  onCopy,
  onCut,
  onPaste,
  onPaneMouseMove,
  onNodesDisable,
  onPaneClick,
}) => {
  const {
    handleNodesDeleteCleanup,
    handleNodeDragOver,
    handleNodeDragStop,
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
    isMainWorkflow,
    onWorkflowAdd,
    onNodesAdd,
    onNodesChange,
    onNodeSettings,
    onEdgesAdd,
    onEdgesChange,
    onNodePickerOpen,
    onCopy,
    onCut,
    onPaste,
    onNodesDisable,
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
      minZoom={0.5}
      proOptions={{ hideAttribution: true }}
      nodeDragThreshold={2}
      snapToGrid
      snapGrid={snapGrid}
      selectionMode={SelectionMode["Partial"]}
      nodes={nodes}
      nodeTypes={nodeTypes}
      edges={edges}
      edgeTypes={readonly ? simpleEdgeTypes : fullEdgeTypes}
      defaultEdgeOptions={defaultEdgeOptions}
      connectionLineComponent={CustomConnectionLine}
      connectionLineStyle={connectionLineStyle}
      isValidConnection={isValidConnection}
      onNodesChange={onNodesChange}
      onEdgesChange={onEdgesChange}
      onNodeDoubleClick={handleNodeSettings}
      onNodeDragStart={handleCloseContextmenu}
      onNodeDragStop={handleNodeDragStop}
      onNodesDelete={handleNodesDeleteCleanup}
      onNodeContextMenu={handleNodeContextMenu}
      onSelectionContextMenu={handleSelectionContextMenu}
      onPaneContextMenu={handlePaneContextMenu}
      onMoveStart={handleCloseContextmenu}
      onDrop={handleNodeDrop}
      onDragOver={handleNodeDragOver}
      onConnect={handleConnect}
      onReconnect={handleReconnect}
      onBeforeDelete={onBeforeDelete}
      onPaneMouseMove={onPaneMouseMove}
      onPaneClick={onPaneClick}>
      <Background
        className="bg-background"
        variant={BackgroundVariant["Dots"]}
        gap={gridSize}
        color="rgba(63, 63, 70, 1)"
      />
      {!readonly && users && currentWorkflowId && (
        <MultiCursor users={users} currentWorkflowId={currentWorkflowId} />
      )}
      {contextMenu && (
        <CanvasContextMenu
          data={contextMenu.data}
          edges={edges}
          allNodes={nodes}
          isMainWorkflow={isMainWorkflow}
          contextMenu={contextMenu}
          onBeforeDelete={onBeforeDelete}
          onNodesChange={onNodesChange}
          onEdgesChange={onEdgesChange}
          onWorkflowOpen={onWorkflowOpen}
          onWorkflowAddFromSelection={onWorkflowAddFromSelection}
          onNodeSettings={onNodeSettings}
          onNodesDeleteCleanup={handleNodesDeleteCleanup}
          onCopy={onCopy}
          onCut={onCut}
          onPaste={onPaste}
          onClose={handleCloseContextmenu}
          onNodesDisable={onNodesDisable}
        />
      )}
    </ReactFlow>
  );
};

export default memo(Canvas);
