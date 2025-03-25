import { Trash, Copy } from "@phosphor-icons/react";
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
  useOnSelectionChange,
} from "@xyflow/react";
import { MouseEvent, memo, useCallback, useState } from "react";

import { useT } from "@flow/lib/i18n";
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
  const t = useT();

  const [selectionMenuPosition, setSelectionMenuPosition] =
    useState<XYPosition | null>(null);
  // TODO: Types below must be fixed and figured out. Right now working on just getting the context menu logic to work etc.
  const [selectedNodes, setSelectedNodes] = useState([]);
  const [selectedEdges, setSelectedEdges] = useState([]);

  const onChange = useCallback(
    ({ nodes, edges }: { nodes: any; edges: any }) => {
      setSelectedNodes(nodes.map((node: any) => node.id));
      setSelectedEdges(edges.map((edge: any) => edge.id));
    },
    [],
  );

  useOnSelectionChange({
    onChange,
  });

  const handleSelectionContextMenu = (event: MouseEvent) => {
    event.preventDefault();
    setSelectionMenuPosition({ x: event.clientX, y: event.clientY });
  };

  const closeSelectionMenu = () => {
    setSelectionMenuPosition(null);
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
      onSelectionContextMenu={handleSelectionContextMenu}>
      <Background
        className="bg-background"
        variant={BackgroundVariant["Lines"]}
        gap={gridSize}
        color="rgba(63, 63, 70, 0.3)"
      />
      {selectionMenuPosition && (
        <div
          className="absolute z-50"
          style={{
            top: selectionMenuPosition.y,
            left: selectionMenuPosition.x,
          }}>
          <div className="min-w-[160px] select-none rounded-md border bg-card p-1 text-popover-foreground shadow-md">
            <div
              className="flex items-center justify-between gap-4 rounded-sm px-2 py-1.5 text-xs hover:bg-accent"
              onClick={() => {
                closeSelectionMenu();
              }}>
              <p>{t("Copy Selected Nodes")}</p>
              <Copy weight="light" />
            </div>
            <div className="-mx-1 my-1 h-px bg-border" />
            <div
              className="flex items-center justify-between gap-4 rounded-sm px-2 py-1.5 text-xs text-destructive hover:bg-accent"
              onClick={() => {
                onNodesChange?.(
                  selectedNodes.map((id) => ({ id, type: "remove" })),
                );
                onEdgesChange?.(
                  selectedEdges.map((id) => ({ id, type: "remove" })),
                );
                closeSelectionMenu();
              }}>
              <p>{t("Delete Selected Nodes")}</p>
              <Trash weight="light" />
            </div>
          </div>
        </div>
      )}

      {selectionMenuPosition && (
        <div
          className="fixed inset-0 z-40"
          onClick={closeSelectionMenu}
          onContextMenu={closeSelectionMenu}
        />
      )}
    </ReactFlow>
  );
};

export default memo(Canvas);
