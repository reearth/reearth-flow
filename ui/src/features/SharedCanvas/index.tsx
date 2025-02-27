import { Array as YArray } from "yjs";

import Canvas from "@flow/features/Canvas";
import { YWorkflow } from "@flow/lib/yjs/types";

import { ParamsPanel, WorkflowTabs } from "../Editor/components";

import useHooks from "./hooks";

type Props = {
  yWorkflows: YArray<YWorkflow>;
  undoTrackerActionWrapper: (callback: () => void) => void;
};

const SharedCanvas: React.FC<Props> = ({
  yWorkflows,
  undoTrackerActionWrapper,
}) => {
  const {
    currentWorkflowId,
    nodes,
    edges,
    openWorkflows,
    locallyLockedNode,
    // isMainWorkflow,
    // hoveredDetails,
    // handleNodeHover,
    // handleEdgeHover,
    handleNodeDoubleClick,
    // handleWorkflowOpen,
    handleNodesChange,
    handleWorkflowClose,
    handleCurrentWorkflowIdChange,
  } = useHooks({ yWorkflows, undoTrackerActionWrapper });
  return (
    <div className="relative flex size-full flex-col">
      <Canvas
        nodes={nodes}
        edges={edges}
        canvasLock
        // onNodeHover={handleNodeHover}
        onNodesChange={handleNodesChange}
        onNodeDoubleClick={handleNodeDoubleClick}
        // onEdgeHover={handleEdgeHover}
      />
      <WorkflowTabs
        className="max-w-full bg-secondary px-1"
        openWorkflows={openWorkflows}
        currentWorkflowId={currentWorkflowId}
        onWorkflowClose={handleWorkflowClose}
        onWorkflowChange={handleCurrentWorkflowIdChange}
      />
      <ParamsPanel selected={locallyLockedNode} />
    </div>
  );
};

export default SharedCanvas;
