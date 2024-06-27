import { ReactFlow, Background, BackgroundVariant, SelectionMode } from "@xyflow/react";
import { MouseEvent } from "react";

import { Edge, Node, Workflow } from "@flow/types";

import { CustomConnectionLine, connectionLineStyle } from "../CustomConnectionLine";
import { edgeTypes } from "../CustomEdge";
import { nodeTypes } from "../Nodes";

import useHooks, { defaultEdgeOptions } from "./hooks";

import "@xyflow/react/dist/style.css";

type Props = {
  workflow?: Workflow;
  onSelect: (nodes?: Node[], edges?: Edge[]) => void;
  onNodeHover: (e: MouseEvent, node?: Node) => void;
  onEdgeHover: (e: MouseEvent, edge?: Edge) => void;
};

const Canvas: React.FC<Props> = ({ workflow, onNodeHover, onEdgeHover }) => {
  const { nodes, edges, onDragOver, onDrop, onNodesChange, onEdgesChange, onConnect } = useHooks({
    workflow,
  });

  return (
    <ReactFlow
      // snapToGrid
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
      selectNodesOnDrag={false}
      selectionMode={SelectionMode["Partial"]}
      nodes={nodes}
      nodeTypes={nodeTypes}
      edges={edges}
      edgeTypes={edgeTypes}
      defaultEdgeOptions={defaultEdgeOptions}
      connectionLineComponent={CustomConnectionLine}
      connectionLineStyle={connectionLineStyle}
      snapGrid={[30, 30]}
      onNodesChange={onNodesChange}
      onEdgesChange={onEdgesChange}
      onNodeMouseEnter={onNodeHover}
      onNodeMouseLeave={onNodeHover}
      onEdgeMouseEnter={onEdgeHover}
      onEdgeMouseLeave={onEdgeHover}
      // onSelectionChange={s => {
      //   onSelect(
      //     s.nodes.filter(n => n.selected),
      //     s.edges.filter(e => e.selected),
      //   );
      // }}
      onConnect={onConnect}
      onDrop={onDrop}
      onDragOver={onDragOver}
      fitViewOptions={{ padding: 0.5 }}
      fitView
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
        gap={30}
        color="rgba(63, 63, 70, 0.5)"
      />
    </ReactFlow>
  );
};

export { Canvas };
