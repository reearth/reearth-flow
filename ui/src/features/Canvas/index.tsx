import { LayersIcon, MaskOffIcon, SizeIcon } from "@radix-ui/react-icons";
import { useState, useCallback } from "react";
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

import { Button } from "@flow/components";
import CustomConnectionLine, {
  connectionLineStyle,
} from "@flow/features/Canvas/components/CustomConnectionLine";
import nodeTypes from "@flow/features/Canvas/components/Nodes";

import "reactflow/dist/style.css";

const initialNodes: Node[] = [
  {
    id: "1",
    type: "reader",
    data: { label: "Reader Node 1" },
    position: { x: 10, y: 1 },
    width: 150,
    height: 150,
  },
  {
    id: "2",
    type: "transformer",
    selected: true,
    data: { label: "Transformer Node 2" },
    position: { x: 115, y: 300 },
  },
  { id: "3", type: "writer", data: { label: "Writer Node 3" }, position: { x: 405, y: 300 } },
  { id: "4", type: "writer", data: { label: "Writer Node 4" }, position: { x: 605, y: 500 } },
  { id: "5", type: "writer", data: { label: "Writer Node 5" }, position: { x: 600, y: 50 } },
  { id: "6", type: "reader", data: { label: "Reader Node 6" }, position: { x: 850, y: 600 } },
];

const initialEdges: Edge[] = [
  { id: "e1-2", source: "1", target: "2" },
  { id: "e1-5", source: "1", target: "5" },
  { id: "e2-3", source: "2", target: "3" },
  { id: "e2-4", source: "2", target: "4" },
];

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

  return (
    <div className="flex-1 mr-1 mb-1 p-1 border border-zinc-700 rounded-sm relative">
      <ReactFlow
        // snapToGrid
        // minZoom={0.7}
        // maxZoom={1}
        // defaultViewport={{ zoom: 0.8, x: 200, y: 200 }}
        // panOnDrag={false}
        nodes={nodes}
        nodeTypes={nodeTypes}
        // nodeDragThreshold={60}
        edges={edges}
        // edgeTypes={edgeTypes}
        defaultEdgeOptions={defaultEdgeOptions}
        connectionLineComponent={CustomConnectionLine}
        connectionLineStyle={connectionLineStyle}
        snapGrid={[30, 30]}
        translateExtent={[
          [-1000, -1000],
          [1000, 1000],
        ]}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
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
      <div className="flex flex-col bg-zinc-800 border border-zinc-600 rounded-md width-[300px] absolute top-3 left-3 text-zinc-400">
        <Button className=" hover:bg-zinc-600 hover:text-zinc-300" variant="ghost" size="sm">
          <LayersIcon />
        </Button>
        <Button className="hover:bg-zinc-600 hover:text-zinc-300" variant="ghost" size="sm">
          <SizeIcon />
        </Button>
        <Button className="hover:bg-zinc-600 hover:text-zinc-300" variant="ghost" size="sm">
          <MaskOffIcon />
        </Button>
        <Button className=" hover:bg-zinc-600 hover:text-zinc-300" variant="ghost" size="sm">
          <LayersIcon />
        </Button>
        <Button className="hover:bg-zinc-600 hover:text-zinc-300" variant="ghost" size="sm">
          <SizeIcon />
        </Button>
        <Button className="hover:bg-zinc-600 hover:text-zinc-300" variant="ghost" size="sm">
          <MaskOffIcon />
        </Button>
        <Button className=" hover:bg-zinc-600 hover:text-zinc-300" variant="ghost" size="sm">
          <LayersIcon />
        </Button>
        <Button className="hover:bg-zinc-600 hover:text-zinc-300" variant="ghost" size="sm">
          <SizeIcon />
        </Button>
        <Button className="hover:bg-zinc-600 hover:text-zinc-300" variant="ghost" size="sm">
          <MaskOffIcon />
        </Button>
        <Button className=" hover:bg-zinc-600 hover:text-zinc-300" variant="ghost" size="sm">
          <LayersIcon />
        </Button>
        <Button className="hover:bg-zinc-600 hover:text-zinc-300" variant="ghost" size="sm">
          <SizeIcon />
        </Button>
        <Button className="hover:bg-zinc-600 hover:text-zinc-300" variant="ghost" size="sm">
          <MaskOffIcon />
        </Button>
      </div>
    </div>
  );
}
