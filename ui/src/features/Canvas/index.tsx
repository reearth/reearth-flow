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
  edgeTypes,
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
  selectedEdgeIds?: string[];
  yDoc?: Doc | null;
  users?: Record<string, AwarenessUser>;
  onWorkflowAdd?: (position?: XYPosition) => void;
  onWorkflowOpen?: (workflowId: string) => void;
  onNodesAdd?: (newNode: Node[]) => void;
  onNodesChange?: (changes: NodeChange<Node>[]) => void;
  onBeforeDelete?: (args: { nodes: Node[] }) => Promise<boolean>;
  onNodeSettings?: (e: MouseEvent | undefined, nodeId: string) => void;
  onNodePickerOpen?: (
    position: XYPosition,
    nodeType?: ActionNodeType,
    isMainWorkflow?: boolean,
  ) => void;
  onEdgesAdd?: (newEdges: Edge[]) => void;
  onEdgesChange?: (changes: EdgeChange[]) => void;
  onCopy?: (node?: Node) => void;
  onCut?: (isCutByShortCut?: boolean, node?: Node) => void;
  onPaste?: () => void;
  onPaneMouseMove?: (event: MouseEvent<Element, globalThis.MouseEvent>) => void;
};

const Canvas: React.FC<Props> = ({
  readonly,
  nodes,
  edges,
  selectedEdgeIds,
  users,
  onWorkflowAdd,
  onWorkflowOpen,
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
}) => {
  const {
    handleNodesDelete,
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
      proOptions={{ hideAttribution: true }}
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
      onNodeContextMenu={handleNodeContextMenu}
      onSelectionContextMenu={handleSelectionContextMenu}
      onPaneContextMenu={handlePaneContextMenu}
      onMoveStart={handleCloseContextmenu}
      onDrop={handleNodeDrop}
      onDragOver={handleNodeDragOver}
      onConnect={handleConnect}
      onReconnect={handleReconnect}
      onBeforeDelete={onBeforeDelete}
      onPaneMouseMove={onPaneMouseMove}>
      <Background
        className="bg-background"
        variant={BackgroundVariant["Dots"]}
        gap={gridSize}
        color="rgba(63, 63, 70, 1)"
      />
      {!readonly && users && <MultiCursor users={users} />}
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
