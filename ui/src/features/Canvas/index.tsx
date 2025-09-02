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
import { MouseEvent, memo, useCallback, useRef } from "react";
import type { Awareness } from "y-protocols/awareness";
import type { Doc } from "yjs";

import {
  isValidConnection,
  CustomConnectionLine,
  edgeTypes,
  connectionLineStyle,
  nodeTypes,
} from "@flow/lib/reactFlow";
import type { ActionNodeType, Edge, Node } from "@flow/types";

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
  yDoc: Doc | null;
  awareness?: Awareness;
  currentUserName?: string;
  onWorkflowAdd?: (position?: XYPosition) => void;
  onWorkflowOpen?: (workflowId: string) => void;
  onNodesAdd?: (newNode: Node[]) => void;
  onNodesChange?: (changes: NodeChange<Node>[]) => void;
  onBeforeDelete?: (args: { nodes: Node[] }) => Promise<boolean>;
  onNodeSettings?: (e: MouseEvent | undefined, nodeId: string) => void;
  onNodePickerOpen?: (position: XYPosition, nodeType?: ActionNodeType) => void;
  onEdgesAdd?: (newEdges: Edge[]) => void;
  onEdgesChange?: (changes: EdgeChange[]) => void;
  onCopy?: (node?: Node) => void;
  onCut?: (isCutByShortCut?: boolean, node?: Node) => void;
  onPaste?: () => void;
};

const Canvas: React.FC<Props> = ({
  readonly,
  nodes,
  edges,
  selectedEdgeIds,
  yDoc,
  awareness,
  currentUserName,
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
}) => {
  const cursorUpdateRef = useRef<
    ((clientX: number, clientY: number) => void) | null
  >(null);

  const handleCursorUpdate = useCallback(
    (updateFn: (clientX: number, clientY: number) => void) => {
      cursorUpdateRef.current = updateFn;
    },
    [],
  );

  const {
    handleNodesDelete,
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

  const handlePaneMouseMove = useCallback((event: MouseEvent) => {
    if (cursorUpdateRef.current) {
      cursorUpdateRef.current(event.clientX, event.clientY);
    }
  }, []);

  return (
    <div className="relative h-full w-full">
      <ReactFlow
        ref={paneRef}
        // Readonly props START
        nodesConnectable={!readonly}
        nodesFocusable={!readonly}
        elementsSelectable={!readonly}
        reconnectRadius={!readonly ? 10 : 0}
        // Readonly props END
        attributionPosition="bottom-left"
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
        onPaneMouseMove={handlePaneMouseMove}>
        <Background
          className="bg-background"
          variant={BackgroundVariant["Dots"]}
          gap={gridSize}
          color="rgba(63, 63, 70, 1)"
        />
        {!readonly && yDoc && awareness && (
          <MultiCursor
            awareness={awareness}
            currentUserName={currentUserName}
            onCursorUpdate={handleCursorUpdate}
          />
        )}
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
    </div>
  );
};

export default memo(Canvas);
