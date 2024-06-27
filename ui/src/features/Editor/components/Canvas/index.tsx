import { ReactFlow, Background, BackgroundVariant, SelectionMode } from "@xyflow/react";
import { Dispatch, MouseEvent, SetStateAction, memo } from "react";

import type { Edge, Node, Workflow } from "@flow/types";

import CustomConnectionLine, { connectionLineStyle } from "../CustomConnectionLine";
import { edgeTypes } from "../CustomEdge";
import { nodeTypes } from "../Nodes";

import useHooks, { defaultEdgeOptions } from "./hooks";

import "@xyflow/react/dist/style.css";

const gridSize = 30;

type Props = {
  workflow?: Workflow;
  lockedNodeIds: string[];
  onNodeLocking: (nodeId: string, setNodes: Dispatch<SetStateAction<Node[]>>) => void;
  onNodeHover: (e: MouseEvent, node?: Node) => void;
  onEdgeHover: (e: MouseEvent, edge?: Edge) => void;
};

const Canvas: React.FC<Props> = ({
  workflow,
  lockedNodeIds,
  onNodeLocking,
  onNodeHover,
  onEdgeHover,
}) => {
  const {
    nodes,
    edges,
    handleNodesChange,
    handleNodesDelete,
    handleNodeDragStop,
    handleNodeDragOver,
    handleNodeDrop,
    handleEdgesChange,
    handleConnect,
  } = useHooks({
    workflow,
    lockedNodeIds,
    onNodeLocking,
  });

  return (
    <ReactFlow
      // minZoom={0.7}
      // maxZoom={1}
      // defaultViewport={{ zoom: 0.8, x: 200, y: 200 }}
      // panOnDrag={false}
      // nodeDragThreshold={60}
      // translateExtent={[
      //   [-1000, -1000],
      //   [1000, 1000],
      // ]}
      // onInit={setReactFlowInstance}
      // selectNodesOnDrag={false}
      // fitViewOptions={{ padding: 0.5 }}
      // fitView
      // snapToGrid
      snapGrid={[gridSize, gridSize]}
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
      onEdgeMouseEnter={onEdgeHover}
      onEdgeMouseLeave={onEdgeHover}
      onConnect={handleConnect}
      onDrop={handleNodeDrop}
      onDragOver={handleNodeDragOver}
      panOnScroll
      proOptions={{ hideAttribution: true }}>
      {/* <MiniMap
      className="bg-zinc-900"
      nodeColor="purple"
      maskStrokeColor="red"
      maskStrokeWidth={3}
    /> */}
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
