import type { Workflow } from "@flow/types";

import { RightPanel, BottomPanel, LeftPanel, Canvas, OverlayUI } from "./components";
import useHooks from "./hooks";

type EditorProps = {
  workflows?: Workflow[];
};

export default function Editor({ workflows }: EditorProps) {
  const {
    currentWorkflow,
    lockedNodeIds,
    locallyLockedNode,
    hoveredDetails,
    handleNodeLocking,
    handleWorkflowChange,
    handleNodeHover,
    handleEdgeHover,
  } = useHooks({ workflows });

  return (
    <div className="relative flex flex-1">
      <LeftPanel data={currentWorkflow} />
      <div className="flex flex-1 flex-col">
        <OverlayUI hoveredDetails={hoveredDetails}>
          <Canvas
            workflow={currentWorkflow}
            lockedNodeIds={lockedNodeIds}
            canvasLock={!!locallyLockedNode}
            onNodeLocking={handleNodeLocking}
            onNodeHover={handleNodeHover}
            onEdgeHover={handleEdgeHover}
          />
        </OverlayUI>
        <BottomPanel
          currentWorkflowId={currentWorkflow?.id}
          onWorkflowChange={handleWorkflowChange}
        />
      </div>
      <RightPanel selected={locallyLockedNode} />
    </div>
  );
}
