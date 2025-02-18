import { Array as YArray, UndoManager as YUndoManager } from "yjs";

import type { YWorkflow } from "@flow/lib/yjs/types";

import {
  BottomPanel,
  Canvas,
  LeftPanel,
  OverlayUI,
  ParamsPanel,
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
    isMainWorkflow,
    hasReader,
    rightPanelContent,
    handleRightPanelOpen,
    handleWorkflowAdd,
    handleWorkflowDeployment,
    handlePanelOpen,
    handleWorkflowClose,
    handleWorkflowChange,
    handleNodesAdd,
    handleNodesChange,
    handleNodeParamsUpdate,
    handleNodeHover,
    handleNodeDoubleClick,
    handleNodePickerOpen,
    handleNodePickerClose,
    handleEdgesAdd,
    handleEdgesChange,
    handleEdgeHover,
    handleWorkflowRedo,
    handleWorkflowUndo,
    handleWorkflowRename,
    handlePanelClose,
  } = useHooks({ yWorkflows, undoManager, undoTrackerActionWrapper });
  // console.log("nodes", nodes);
  // console.log("edges", edges);
  return (
    <div className="flex h-screen flex-col">
      <div className="relative flex flex-1">
        <LeftPanel
          nodes={nodes}
          isOpen={openPanel === "left"}
          onOpen={handlePanelOpen}
          onNodesAdd={handleNodesAdd}
          isMainWorkflow={isMainWorkflow}
          hasReader={hasReader}
          onNodeDoubleClick={handleNodeDoubleClick}
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
            onNodesAdd={handleNodesAdd}
            onNodePickerClose={handleNodePickerClose}
            onRightPanelOpen={handleRightPanelOpen}
            isMainWorkflow={isMainWorkflow}
            hasReader={hasReader}>
            <Canvas
              nodes={nodes}
              edges={edges}
              canvasLock={!!locallyLockedNode}
              onWorkflowAdd={handleWorkflowAdd}
              onNodesAdd={handleNodesAdd}
              onNodesChange={handleNodesChange}
              onNodeHover={handleNodeHover}
              onNodeDoubleClick={handleNodeDoubleClick}
              onNodePickerOpen={handleNodePickerOpen}
              onEdgesAdd={handleEdgesAdd}
              onEdgesChange={handleEdgesChange}
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
          contentType={rightPanelContent}
          onClose={() => handleRightPanelOpen(undefined)}
        />
        <ParamsPanel
          selected={locallyLockedNode}
          onParamsSubmit={handleNodeParamsUpdate}
          onClose={handlePanelClose}
        />
      </div>
    </div>
  );
}
