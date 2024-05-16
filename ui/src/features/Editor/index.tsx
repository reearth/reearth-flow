import { useState, useMemo, useEffect, useCallback, MouseEvent } from "react";
import { Node, Edge, useNodesState, useEdgesState } from "reactflow";

import LeftPanel from "@flow/features/LeftPanel";
import RightPanel from "@flow/features/RightPanel";
import type { Workflow } from "@flow/types";

import BottomPanel from "../BottomPanel";

import { Canvas, OverlayUI } from "./components";

type EditorProps = {
  workflow?: Workflow;
};

export default function Editor({ workflow }: EditorProps) {
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

  return (
    <div className="flex flex-1 relative">
      <LeftPanel data={workflow} />
      <div className="flex flex-col flex-1">
        <OverlayUI hoveredDetails={hoveredDetails}>
          <Canvas
            workflow={workflow}
            nodes={nodes}
            edges={edges}
            setNodes={setNodes}
            setEdges={setEdges}
            onNodeHover={handleNodeHover}
            onEdgeHover={handleEdgeHover}
          />
        </OverlayUI>
        <BottomPanel />
      </div>
      <RightPanel selected={selected.nodes} />
    </div>
  );
}
