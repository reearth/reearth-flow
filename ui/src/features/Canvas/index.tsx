import { useState, useCallback, MouseEvent, useMemo, useEffect } from "react";
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
  ReactFlowProvider,
  useNodesState,
  useEdgesState,
  SelectionMode,
} from "reactflow";

import ActionBar from "@flow/features/Actionbar";
import {
  Infobar,
  nodeTypes,
  CustomConnectionLine,
  connectionLineStyle,
  Toolbox,
} from "@flow/features/Canvas/components";

import "reactflow/dist/style.css";

import { useDialogAtom } from "@flow/stores";

import useDnd from "./useDnd";

type CanvasProps = {
  leftArea?: React.ReactNode;
  workflow?: {
    nodes?: Node[];
    edges?: Edge[];
  };
};

// const edgeTypes: EdgeTypes = {
//   floating: FloatingEdge,
// };

const defaultEdgeOptions: DefaultEdgeOptions = {
  // stroke style for unsure (normal) state: rgb(234, 179, 8) bg-yellow-500
  // stroke style for success state: rgb(22, 163, 74) bg-green (after running workflow)
  // stroke style for error state: "#7f1d1d" (after running workflow)
  style: { strokeWidth: 2, stroke: "rgb(234, 179, 8)" },
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

export default function Canvas({ workflow, leftArea }: CanvasProps) {
  const [nodes, setNodes] = useNodesState(workflow?.nodes ?? []);
  const [edges, setEdges] = useEdgesState(workflow?.edges ?? []);

  const [currentDialogType, setDialogType] = useDialogAtom();

  const selected = useMemo(() => {
    const selectedNodes = nodes.filter(node => node.selected);
    const selectedEdges = edges.filter(edge => edge.selected);
    return { nodes: selectedNodes, edges: selectedEdges };
  }, [nodes, edges]);

  console.log("selected", selected);

  const [hoveredDetails, setHoveredDetails] = useState<Node | Edge | undefined>();

  const { onDragOver, onDrop, setReactFlowInstance } = useDnd({ setNodes });

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

  // useEffect(() => {
  //   console.log("hoveredDetails", hoveredDetails);
  // }, [hoveredDetails]);

  useEffect(() => {
    console.log("hi");
    if (!workflow && currentDialogType !== "welcome-init") {
      setDialogType("welcome-init");
    } else if (workflow && currentDialogType === "welcome-init") {
      setDialogType(undefined);
    }
  }, [workflow, currentDialogType, setDialogType]);

  return (
    <div className="flex-1 m-1 rounded-sm relative">
      <ReactFlowProvider>
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
          selectionMode={SelectionMode["Partial"]}
          nodes={nodes}
          nodeTypes={nodeTypes}
          edges={edges}
          onInit={setReactFlowInstance}
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
          <Background variant={BackgroundVariant["Lines"]} gap={30} color="rgb(39 39 42)" />
        </ReactFlow>
        <div className="absolute top-1 right-1">
          <ActionBar />
        </div>
        {leftArea && (
          <div className="absolute left-1 top-1 bottom-1 flex flex-shrink-0 gap-2">
            {leftArea}
            <Toolbox className="self-start" />
          </div>
        )}
        <Infobar
          className="absolute bottom-1 left-[50%] translate-x-[-50%]"
          hoveredDetails={hoveredDetails}
        />
        {/* <BottomPanel className="absolute right-1 bottom-1" /> */}
      </ReactFlowProvider>
    </div>
  );
}
