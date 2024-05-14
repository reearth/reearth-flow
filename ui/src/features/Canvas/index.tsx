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
  useNodesState,
  useEdgesState,
  SelectionMode,
  useReactFlow,
} from "reactflow";

import {
  Infobar,
  nodeTypes,
  CustomConnectionLine,
  connectionLineStyle,
  Toolbox,
  Breadcrumb,
} from "@flow/features/Canvas/components";
import CanvasActionBar from "@flow/features/Canvas/components/CanvasActionbar";
import LeftPanel from "@flow/features/LeftPanel";
import RightPanel from "@flow/features/RightPanel";
import type { Workflow } from "@flow/types";

import BottomPanel from "../BottomPanel";

import ActionBar from "./components/Actionbar";
import { edgeTypes } from "./components/CustomEdge";
import useDnd from "./useDnd";

import "reactflow/dist/style.css";

type CanvasProps = {
  workflow?: Workflow;
};

// const edgeTypes: EdgeTypes = {
//   floating: FloatingEdge,
// };

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

export default function Canvas({ workflow }: CanvasProps) {
  const reactFlowInstance = useReactFlow();
  console.log("reactFlowInstance", reactFlowInstance);
  // console.log("reactFlowInstance to object", reactFlowInstance.toObject());

  const [currentWorkflowId, setCurrentWorkflowId] = useState<string>(workflow?.id ?? "");

  const [nodes, setNodes] = useNodesState(workflow?.nodes ?? []);
  const [edges, setEdges] = useEdgesState(workflow?.edges ?? []);

  useEffect(() => {
    if (workflow?.id !== currentWorkflowId) {
      setNodes(workflow?.nodes ?? []);
      setEdges(workflow?.edges ?? []);
      setCurrentWorkflowId(workflow?.id ?? "");
    }
  }, [currentWorkflowId, workflow, setNodes, setEdges]);

  const selected = useMemo(() => {
    const selectedNodes = nodes.filter(node => node.selected);
    const selectedEdges = edges.filter(edge => edge.selected);
    return { nodes: selectedNodes, edges: selectedEdges };
  }, [nodes, edges]);

  console.log("selected", selected);

  const [hoveredDetails, setHoveredDetails] = useState<Node | Edge | undefined>();

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
    if (workflow) {
      setNodes(workflow.nodes ?? []);
      setEdges(workflow.edges ?? []);
    }
  }, [workflow, setNodes, setEdges]);

  return (
    <div className="flex flex-1">
      <LeftPanel data={workflow} />
      <div className="relative flex flex-col flex-1">
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
          <Background
            className="bg-zinc-900/50"
            variant={BackgroundVariant["Lines"]}
            gap={30}
            color="rgba(63, 63, 70, 0.3)"
          />
        </ReactFlow>
        <Breadcrumb />
        <div className="absolute left-2 top-2 bottom-1 flex flex-shrink-0 gap-2 pointer-events-none [&>*]:pointer-events-auto">
          <Toolbox className="self-start" />
        </div>
        <div className="absolute top-1 right-1">
          <ActionBar />
        </div>
        <div className="absolute bottom-12 right-2">
          <CanvasActionBar />
        </div>
        <Infobar
          className="absolute bottom-[42px] left-[50%] translate-x-[-50%]"
          hoveredDetails={hoveredDetails}
        />
        <BottomPanel />
      </div>
      <RightPanel selected={selected.nodes} />
    </div>
  );
}
