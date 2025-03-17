import { useMemo } from "react";
import { Doc, Map as YMap, UndoManager as YUndoManager } from "yjs";

import { LoadingSplashscreen } from "@flow/components";
import Canvas from "@flow/features/Canvas";
import { YWorkflow } from "@flow/lib/yjs/types";
import { useIsReverting } from "@flow/stores";

import {
  BottomBar,
  LeftPanel,
  OverlayUI,
  ParamsPanel,
  RightPanel,
} from "./components";
import { EditorContextType, EditorProvider } from "./editorContext";
import useHooks from "./hooks";

type Props = {
  yWorkflows: YMap<YWorkflow>;
  undoManager: YUndoManager | null;
  undoTrackerActionWrapper: (callback: () => void) => void;
  yDoc: Doc | null;
};

export default function Editor({
  yWorkflows,
  undoManager,
  undoTrackerActionWrapper,
  yDoc,
}: Props) {
  const {
    currentWorkflowId,
    openWorkflows,
    currentProject,
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
    handleProjectShare,
    handlePanelOpen,
    handleWorkflowClose,
    handleWorkflowChange,
    handleNodesAdd,
    handleNodesChange,
    handleNodeDataUpdate,
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
    handleDebugRunStart,
    handleDebugRunStop,
    handleLayoutChange,
  } = useHooks({ yWorkflows, undoManager, undoTrackerActionWrapper });

  const editorContext = useMemo(
    (): EditorContextType => ({
      onNodesChange: handleNodesChange,
      onSecondaryNodeAction: handleNodeDoubleClick,
    }),
    [handleNodesChange, handleNodeDoubleClick],
  );
  const [isReverting] = useIsReverting();
  return (
    <div className="flex h-screen flex-col">
      <div className="relative flex flex-1">
        <EditorProvider value={editorContext}>
          <LeftPanel
            nodes={nodes}
            isOpen={openPanel === "left"}
            onOpen={handlePanelOpen}
            onNodesAdd={handleNodesAdd}
            isMainWorkflow={isMainWorkflow}
            hasReader={hasReader}
            onNodesChange={handleNodesChange}
            onNodeDoubleClick={handleNodeDoubleClick}
            selected={locallyLockedNode}
          />
          <div className="flex flex-1 flex-col">
            <OverlayUI
              hoveredDetails={hoveredDetails}
              nodePickerOpen={nodePickerOpen}
              allowedToDeploy={allowedToDeploy}
              canUndo={canUndo}
              canRedo={canRedo}
              isMainWorkflow={isMainWorkflow}
              hasReader={hasReader}
              onWorkflowDeployment={handleWorkflowDeployment}
              onProjectShare={handleProjectShare}
              onNodesAdd={handleNodesAdd}
              onNodePickerClose={handleNodePickerClose}
              onRightPanelOpen={handleRightPanelOpen}
              onWorkflowUndo={handleWorkflowUndo}
              onWorkflowRedo={handleWorkflowRedo}
              onDebugRunStart={handleDebugRunStart}
              onDebugRunStop={handleDebugRunStop}
              onLayoutChange={handleLayoutChange}>
              {!isReverting ? (
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
              ) : (
                <LoadingSplashscreen />
              )}
            </OverlayUI>
            <BottomBar
              currentWorkflowId={currentWorkflowId}
              openWorkflows={openWorkflows}
              onWorkflowClose={handleWorkflowClose}
              onWorkflowChange={handleWorkflowChange}
              onWorkflowRename={handleWorkflowRename}
            />
          </div>
          <RightPanel
            contentType={rightPanelContent}
            onClose={() => handleRightPanelOpen(undefined)}
            project={currentProject}
            yDoc={yDoc}
          />
          <ParamsPanel
            selected={locallyLockedNode}
            onDataSubmit={handleNodeDataUpdate}
          />
        </EditorProvider>
      </div>
    </div>
  );
}
