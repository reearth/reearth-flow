import { useState, useCallback } from "react";
import ReactFlow, {
  addEdge,
  applyNodeChanges,
  applyEdgeChanges,
  Node,
  Edge,
  OnNodesChange,
  OnEdgesChange,
  MiniMap,
  OnConnect,
  NodeTypes,
  DefaultEdgeOptions,
  NodeProps,
  Background,
  BackgroundVariant,
  Handle,
  Position,
} from "reactflow";

import "reactflow/dist/style.css";

type NodeData = {
  label: number;
};

export type CustomNode = Node<NodeData>;

function CustomTextUpdaterNode({ data }: NodeProps<NodeData>) {
  console.log("D", data);
  // const onChange = useCallback(
  //   (evt: any) => {
  //     console.log("EVT", evt.target.value);
  //     console.log("data", data);
  //   },
  //   [data],
  // );
  const handleStyle = { left: 10 };
  return (
    <>
      <Handle type="target" position={Position.Top} />
      <div className="bg-zinc-700 text-zinc-300 border border-zinc-600 rounded-sm p-[8px] w-[150px] h-[80px]">
        <label htmlFor="text">{data.label}</label>
        <p>............</p>
        {/* <input id="text" name="text" onChange={onChange} className="nodrag" /> */}
      </div>
      <Handle type="source" position={Position.Bottom} id="a" />
      <Handle type="source" position={Position.Bottom} id="b" style={handleStyle} />
    </>
  );
}

const initialNodes: Node[] = [
  {
    id: "1",
    type: "custom",
    data: { label: "Node 1" },
    position: { x: 100, y: 1 },
    width: 150,
    height: 150,
  },
  {
    id: "2",
    type: "custom",
    selected: true,
    data: { label: "Node 2" },
    position: { x: 115, y: 300 },
  },
  { id: "3", type: "custom", data: { label: "Node 3" }, position: { x: 405, y: 300 } },
  { id: "4", type: "custom", data: { label: "Node 4" }, position: { x: 605, y: 500 } },
  { id: "5", type: "custom", data: { label: "Node 5" }, position: { x: 600, y: 50 } },
  { id: "6", type: "custom", data: { label: "Node 6" }, position: { x: 650, y: 300 } },
];

const initialEdges: Edge[] = [
  { id: "e1-2", source: "1", target: "2" },
  { id: "e2-3", source: "2", target: "3" },
  { id: "e2-4", source: "2", target: "4" },
];

const defaultEdgeOptions: DefaultEdgeOptions = {
  // animated: true,
};

const nodeTypes: NodeTypes = {
  custom: CustomTextUpdaterNode,
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
    <ReactFlow
      nodes={nodes}
      // snapToGrid
      snapGrid={[30, 30]}
      translateExtent={[
        [0, 0],
        [2000, 2000],
      ]}
      edges={edges}
      onNodesChange={onNodesChange}
      onEdgesChange={onEdgesChange}
      onConnect={onConnect}
      // maxZoom={1}
      fitView
      // fitViewOptions={}
      // nodeDragThreshold={100}
      // minZoom={0.7}
      // defaultViewport={{ zoom: 0.8, x: 200, y: 200 }}
      // panOnDrag={false}
      defaultEdgeOptions={defaultEdgeOptions}
      nodeTypes={nodeTypes}
      proOptions={{ hideAttribution: true }}>
      <MiniMap
        className="bg-zinc-900"
        nodeColor="purple"
        maskStrokeColor="red"
        maskStrokeWidth={3}
      />
      <Background variant={BackgroundVariant["Lines"]} gap={30} color="rgb(39 39 42)" />
    </ReactFlow>
  );
}
