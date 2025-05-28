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
    openNode,
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
    handleWorkflowOpen,
    handleWorkflowClose,
    handleWorkflowChange,
    handleNodesAdd,
    handleNodesChange,
    handleNodeDataUpdate,
    handleNodeHover,
    handleNodeSettings,
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
      onNodeSettings: handleNodeSettings,
    }),
    [handleNodesChange, handleNodeSettings],
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
                onWorkflowAdd={handleWorkflowAdd}
                onWorkflowOpen={handleWorkflowOpen}
                onNodesAdd={handleNodesAdd}
                onNodesChange={handleNodesChange}
                onNodeHover={handleNodeHover}
                onNodeSettings={handleNodeSettings}
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
          {openNode && (
            <ParamsPanel
              openNode={openNode}
              onDataSubmit={handleNodeDataUpdate}
              onWorkflowRename={handleWorkflowRename}
            />
          )}
        </div>
      </EditorProvider>
    </div>
  );
}
