import { Array as YArray, UndoManager as YUndoManager } from "yjs";

import { YWorkflow } from "@flow/lib/yjs/utils";

import {
  BottomPanel,
  Canvas,
  LeftPanel,
  OverlayUI,
  RightPanel,
} from "./components";
import useHooks from "./hooks";

type Props = {
  yWorkflows: YArray<YWorkflow>;
  undoManager: YUndoManager | null;
  undoTrackerActionWrapper: (callback: () => void) => void;
};

export default function Editor({
  yWorkflows,
  undoManager,
  undoTrackerActionWrapper,
}: Props) {
  const {
    currentWorkflowId,
    openWorkflows,
    nodes,
    edges,
    // lockedNodeIds,
    locallyLockedNode,
    hoveredDetails,
    nodePickerOpen,
    openPanel,
    canUndo,
    canRedo,
    allowedToDeploy,
    handleWorkflowAdd,
    handleWorkflowDeployment,
    handlePanelOpen,
    handleWorkflowClose,
    handleWorkflowChange,
    handleYNodesAdd,
    handleYNodesChange,
    handleNodeParamsUpdate,
    handleNodeHover,
    handleNodeDoubleClick,
    handleNodePickerOpen,
    handleNodePickerClose,
    handleEdgeSelection,
    handleEdgesUpdate,
    handleEdgeHover,
    handleWorkflowRedo,
    handleWorkflowUndo,
    handleWorkflowRename,
    isMainWorkflow,
    hasReader,
  } = useHooks({ yWorkflows, undoManager, undoTrackerActionWrapper });
  return (
    <div className="flex h-screen flex-col">
      <div className="relative flex flex-1">
        <LeftPanel
          nodes={nodes}
          isOpen={openPanel === "left" && !locallyLockedNode}
          onOpen={handlePanelOpen}
          onNodesAdd={handleYNodesAdd}
          isMainWorkflow={isMainWorkflow}
          hasReader={hasReader}
        />
        <div className="flex flex-1 flex-col">
          <OverlayUI
            hoveredDetails={hoveredDetails}
            nodePickerOpen={nodePickerOpen}
            allowedToDeploy={allowedToDeploy}
            canUndo={canUndo}
            canRedo={canRedo}
            onWorkflowDeployment={handleWorkflowDeployment}
            onWorkflowUndo={handleWorkflowUndo}
            onWorkflowRedo={handleWorkflowRedo}
            onNodesAdd={handleYNodesAdd}
            onNodePickerClose={handleNodePickerClose}
            isMainWorkflow={isMainWorkflow}
            hasReader={hasReader}>
            <Canvas
              nodes={nodes}
              edges={edges}
              canvasLock={!!locallyLockedNode}
              onWorkflowAdd={handleWorkflowAdd}
              onNodesAdd={handleYNodesAdd}
              onNodesChange={handleYNodesChange}
              onNodeHover={handleNodeHover}
              onEdgeSelection={handleEdgeSelection}
              onNodeDoubleClick={handleNodeDoubleClick}
              onNodePickerOpen={handleNodePickerOpen}
              onEdgesUpdate={handleEdgesUpdate}
              onEdgeHover={handleEdgeHover}
            />
          </OverlayUI>
          <BottomPanel
            currentWorkflowId={currentWorkflowId}
            openWorkflows={openWorkflows}
            isOpen={openPanel === "bottom" && !locallyLockedNode}
            onOpen={handlePanelOpen}
            onWorkflowClose={handleWorkflowClose}
            onWorkflowChange={handleWorkflowChange}
            onWorkflowRename={handleWorkflowRename}
          />
        </div>
        <RightPanel
          selected={locallyLockedNode}
          onParamsSubmit={handleNodeParamsUpdate}
        />
      </div>
    </div>
  );
}
