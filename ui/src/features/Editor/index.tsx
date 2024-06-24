import { useState, useCallback, MouseEvent } from "react";

import LeftPanel from "@flow/features/LeftPanel";
import RightPanel from "@flow/features/RightPanel";
import type { Edge, Node, Workflow } from "@flow/types";

import BottomPanel from "../BottomPanel";

import { Canvas, OverlayUI } from "./components";

type EditorProps = {
  workflows?: Workflow[];
};

export default function Editor({ workflows }: EditorProps) {
  const [selected, setSelected] = useState<{ nodes: Node[]; edges: Edge[] }>({
    nodes: [],
    edges: [],
  });
  const [currentWorkflow, setCurrentWorkflow] = useState<Workflow | undefined>(workflows?.[0]);

  const handleWorkflowChange = (workflowId?: string) => {
    if (!workflowId) return setCurrentWorkflow(workflows?.[0]);
    const workflow = workflows?.find(w => w.id === workflowId);
    setCurrentWorkflow(workflow);
  };

  const handleSelect = (nodes?: Node[], edges?: Edge[]) => {
    setSelected({ nodes: nodes ?? [], edges: edges ?? [] });
  };

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
      <LeftPanel data={currentWorkflow} />
      <div className="flex flex-col flex-1">
        <OverlayUI hoveredDetails={hoveredDetails}>
          <Canvas
            workflow={currentWorkflow}
            onSelect={handleSelect}
            onNodeHover={handleNodeHover}
            onEdgeHover={handleEdgeHover}
          />
        </OverlayUI>
        <BottomPanel
          currentWorkflowId={currentWorkflow?.id}
          onWorkflowChange={handleWorkflowChange}
        />
      </div>
      <RightPanel selected={selected.nodes} />
    </div>
  );
}
