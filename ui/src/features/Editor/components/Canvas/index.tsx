import { Dispatch, MouseEvent, SetStateAction, useCallback, useEffect } from "react";
import ReactFlow, {
  Background,
  BackgroundVariant,
  DefaultEdgeOptions,
  Edge,
  Node,
  OnConnect,
  OnEdgesChange,
  OnNodesChange,
  SelectionMode,
  addEdge,
  applyEdgeChanges,
  applyNodeChanges,
  useReactFlow,
} from "reactflow";

import { EdgeData, NodeData, Workflow } from "@flow/types";

import { CustomConnectionLine, connectionLineStyle } from "../CustomConnectionLine";
import { edgeTypes } from "../CustomEdge";
import { nodeTypes } from "../Nodes";

import useDnd from "./useDnd";

import "reactflow/dist/style.css";

type Props = {
  workflow?: Workflow;
  nodes?: Node[];
  edges?: Edge[];
  setNodes: Dispatch<SetStateAction<Node<NodeData, string | undefined>[]>>;
  setEdges: Dispatch<SetStateAction<Edge<EdgeData>[]>>;
  onNodeHover: (e: MouseEvent, node?: Node) => void;
  onEdgeHover: (e: MouseEvent, edge?: Edge) => void;
};

const defaultEdgeOptions: DefaultEdgeOptions = {
  // stroke style for unsure (normal) state: rgb(234, 179, 8) bg-yellow-500
  // stroke style for success state: rgb(22, 163, 74) bg-green (after running workflow)
  // stroke style for error state: "#7f1d1d" (after running workflow)
  // style: { strokeWidth: 2, stroke: "rgb(234, 179, 8)" },
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

const Canvas: React.FC<Props> = ({
  workflow,
  nodes,
  edges,
  setNodes,
  setEdges,
  onNodeHover,
  onEdgeHover,
}) => {
  const reactFlowInstance = useReactFlow();
  console.log("reactFlowInstance", reactFlowInstance);

  const { onDragOver, onDrop } = useDnd({ setNodes });

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

  useEffect(() => {
    if (workflow) {
      setNodes(workflow.nodes ?? []);
      setEdges(workflow.edges ?? []);
    }
  }, [workflow, setNodes, setEdges]);

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
