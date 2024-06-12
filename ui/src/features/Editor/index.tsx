import { useState, useCallback, MouseEvent } from "react";

import LeftPanel from "@flow/features/LeftPanel";
import RightPanel from "@flow/features/RightPanel";
import type { Edge, Node, Workflow } from "@flow/types";

import BottomPanel from "../BottomPanel";

import { Canvas, OverlayUI } from "./components";

type EditorProps = {
  workflow?: Workflow;
};

export default function Editor({ workflow }: EditorProps) {
  const [selected, setSelected] = useState<{ nodes: Node[]; edges: Edge[] }>({
    nodes: [],
    edges: [],
  });

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
      <LeftPanel data={workflow} />
      <div className="flex flex-col flex-1">
        <OverlayUI hoveredDetails={hoveredDetails}>
          <Canvas
            workflow={workflow}
            onSelect={handleSelect}
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
