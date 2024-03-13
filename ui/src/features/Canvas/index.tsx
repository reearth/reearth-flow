import { useState, useCallback, MouseEvent, useEffect } from "react";
import ReactFlow, {
  addEdge,
  applyNodeChanges,
  applyEdgeChanges,
  Node,
  Edge,
  OnNodesChange,
  OnEdgesChange,
  // MiniMap,
  OnConnect,
  Background,
  BackgroundVariant,
  DefaultEdgeOptions,
} from "reactflow";

import {
  Infobar,
  Toolbox,
  nodeTypes,
  CustomConnectionLine,
  connectionLineStyle,
} from "@flow/features/Canvas/components";

import "reactflow/dist/style.css";
import { initialEdges, initialNodes } from "./mockData";

// const edgeTypes: EdgeTypes = {
//   floating: FloatingEdge,
// };

const defaultEdgeOptions: DefaultEdgeOptions = {
  style: { strokeWidth: 2, stroke: "#7f1d1d" },
  // type: "floating",
  //   markerEnd: {
  //     type: MarkerType.ArrowClosed,
  //     color: "red",
  //   },
  //   markerStart: {
  //     type: MarkerType.ArrowClosed,
  //     color: "green",
  //   },
  // animated: true,
};

export default function Canvas() {
  const [nodes, setNodes] = useState<Node[]>(initialNodes);
  const [edges, setEdges] = useState<Edge[]>(initialEdges);

  const [hoveredDetails, setHoveredDetails] = useState<Node | Edge | undefined>();

  const onNodesChange: OnNodesChange = useCallback(
    changes => {
      setNodes(nds => applyNodeChanges(changes, nds));
      console.log("CHAGNES", changes);
    },
    [setNodes],
  );

  const onEdgesChange: OnEdgesChange = useCallback(
    changes => setEdges(eds => applyEdgeChanges(changes, eds)),
    [setEdges],
  );

  const onConnect: OnConnect = useCallback(
    connection => setEdges(eds => addEdge(connection, eds)),
    [setEdges],
  );

  const handleNodeHover = useCallback(
    (e: MouseEvent, node?: Node) => {
      if (e.type === "mouseleave" && hoveredDetails) {
        setHoveredDetails(undefined);
      } else {
        setHoveredDetails(node);
      }
    },
    [hoveredDetails],
  );

  const handleEdgeHover = useCallback(
    (e: MouseEvent, edge?: Edge) => {
      if (e.type === "mouseleave" && hoveredDetails) {
        setHoveredDetails(undefined);
      } else {
        setHoveredDetails(edge);
      }
    },
    [hoveredDetails],
  );

  useEffect(() => {
    console.log("hoveredDetails", hoveredDetails);
  }, [hoveredDetails]);

  return (
    <div className="flex-1 mb-1 p-1 border border-zinc-700 rounded-sm relative">
      <ReactFlow
        // snapToGrid
        // minZoom={0.7}
        // maxZoom={1}
        // defaultViewport={{ zoom: 0.8, x: 200, y: 200 }}
        // panOnDrag={false}
        // nodeDragThreshold={60}
        // edgeTypes={edgeTypes}
        // translateExtent={[
        //   [-1000, -1000],
        //   [1000, 1000],
        // ]}
        nodes={nodes}
        nodeTypes={nodeTypes}
        edges={edges}
        defaultEdgeOptions={defaultEdgeOptions}
        connectionLineComponent={CustomConnectionLine}
        connectionLineStyle={connectionLineStyle}
        snapGrid={[30, 30]}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onNodeMouseEnter={handleNodeHover}
        onNodeMouseLeave={handleNodeHover}
        onEdgeMouseEnter={handleEdgeHover}
        onEdgeMouseLeave={handleEdgeHover}
        onConnect={onConnect}
        fitView
        panOnScroll
        proOptions={{ hideAttribution: true }}>
        {/* <MiniMap
          className="bg-zinc-900"
          nodeColor="purple"
          maskStrokeColor="red"
          maskStrokeWidth={3}
        /> */}
        <Background variant={BackgroundVariant["Lines"]} gap={30} color="rgb(39 39 42)" />
      </ReactFlow>
      <Toolbox />
      <Infobar hoveredDetails={hoveredDetails} />
    </div>
  );
}
