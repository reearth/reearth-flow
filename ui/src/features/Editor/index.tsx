import { useMemo } from "react";
import { Doc, Map as YMap, UndoManager as YUndoManager } from "yjs";

import Canvas from "@flow/features/Canvas";
import { YWorkflow } from "@flow/lib/yjs/types";

import { TopBar, OverlayUI, ParamsPanel, RightPanel } from "./components";
import { EditorContextType, EditorProvider } from "./editorContext";
import useHooks from "./hooks";

type Props = {
  yWorkflows: YMap<YWorkflow>;
  undoManager: YUndoManager | null;
  undoTrackerActionWrapper: (
    callback: () => void,
    originPrepend?: string,
  ) => void;
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
    isSubworkflow,
    openWorkflows,
    currentProject,
    nodes,
    edges,
    selectedEdgeIds,
    // lockedNodeIds,
    locallyLockedNode,
    hoveredDetails,
    nodePickerOpen,
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
    handleCopy,
    handleCut,
    handlePaste,
  } = useHooks({ yWorkflows, undoManager, undoTrackerActionWrapper });

  const editorContext = useMemo(
    (): EditorContextType => ({
      onNodesChange: handleNodesChange,
      onSecondaryNodeAction: handleNodeDoubleClick,
    }),
    [handleNodesChange, handleNodeDoubleClick],
  );

  return (
    <div className="flex h-screen flex-col">
      <EditorProvider value={editorContext}>
        <TopBar
          currentWorkflowId={currentWorkflowId}
          openWorkflows={openWorkflows}
          allowedToDeploy={allowedToDeploy}
          onProjectShare={handleProjectShare}
          onRightPanelOpen={handleRightPanelOpen}
          onWorkflowDeployment={handleWorkflowDeployment}
          onWorkflowClose={handleWorkflowClose}
          onWorkflowChange={handleWorkflowChange}
          onDebugRunStart={handleDebugRunStart}
          onDebugRunStop={handleDebugRunStop}
        />
        <div className="relative flex flex-1">
          <div className="flex flex-1 flex-col">
            <OverlayUI
              hoveredDetails={hoveredDetails}
              nodePickerOpen={nodePickerOpen}
              canUndo={canUndo}
              canRedo={canRedo}
              isMainWorkflow={isMainWorkflow}
              hasReader={hasReader}
              onNodesAdd={handleNodesAdd}
              onNodePickerClose={handleNodePickerClose}
              onWorkflowUndo={handleWorkflowUndo}
              onWorkflowRedo={handleWorkflowRedo}
              onLayoutChange={handleLayoutChange}>
              <Canvas
                isSubworkflow={isSubworkflow}
                nodes={nodes}
                edges={edges}
                selectedEdgeIds={selectedEdgeIds}
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
                onCopy={handleCopy}
                onCut={handleCut}
                onPaste={handlePaste}
              />
            </OverlayUI>
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
            onWorkflowRename={handleWorkflowRename}
          />
        </div>
      </EditorProvider>
    </div>
  );
}
