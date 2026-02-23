import { useMemo } from "react";
import type { Awareness } from "y-protocols/awareness";
import { Doc, Map as YMap, UndoManager as YUndoManager } from "yjs";

import Canvas from "@flow/features/Canvas";
import { YWorkflow } from "@flow/lib/yjs/types";

import { OverlayUI, ParamsDialog, NodeDeletionDialog } from "./components";
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
  yAwareness: Awareness;
};

export default function Editor({
  yWorkflows,
  undoManager,
  undoTrackerActionWrapper,
  yDoc,
  yAwareness,
}: Props) {
  const {
    currentWorkflowId,
    currentYWorkflow,
    openWorkflows,
    currentProject,
    self,
    users,
    nodes,
    edges,
    openNode,
    nodePickerOpen,
    canUndo,
    canRedo,
    allowedToDeploy,
    isMainWorkflow,
    deferredDeleteRef,
    showBeforeDeleteDialog,
    isSaving,
    spotlightUserClientId,
    spotlightUser,
    activeUsersDebugRuns,
    rawWorkflows,
    customDebugRunWorkflowVariables,
    showSearchPanel,
    openNodePickerViaShortcut,
    handleDebugRunVariableValueChange,
    loadExternalDebugJob,
    handleWorkflowAdd,
    handleWorkflowDeployment,
    handleProjectShare,
    handleCurrentProjectExport,
    handleWorkflowOpen,
    handleWorkflowClose,
    handleWorkflowChange,
    handleNodesAdd,
    handleNodesChange,
    handleBeforeDeleteNodes,
    handleDeleteDialogClose,
    handleNodesDataUpdate,
    handleNodeSettings,
    handleOpenNode,
    handleNodePickerOpen,
    handleNodePickerClose,
    handleEdgesAdd,
    handleEdgesChange,
    handleWorkflowRedo,
    handleWorkflowUndo,
    handleWorkflowRename,
    handleWorkflowAddFromSelection,
    handleDebugRunStart,
    handleFromSelectedNodeDebugRunStart,
    handleDebugRunStop,
    handleLayoutChange,
    handleCopy,
    handleCut,
    handlePaste,
    handleProjectSnapshotSave,
    handlePaneMouseMove,
    handleSpotlightUserSelect,
    handleSpotlightUserDeselect,
    handleNodesDisable,
    handlePaneClick,
    setShowSearchPanel,
    selectedNodeIds,
  } = useHooks({
    yDoc,
    yWorkflows,
    yAwareness,
    undoManager,
    undoTrackerActionWrapper,
  });

  const editorContext = useMemo(
    (): EditorContextType => ({
      onNodesChange: handleNodesChange,
      onNodeSettings: handleNodeSettings,
      currentYWorkflow,
      undoTrackerActionWrapper,
    }),
    [
      handleNodesChange,
      handleNodeSettings,
      currentYWorkflow,
      undoTrackerActionWrapper,
    ],
  );

  return (
    <div className="flex h-screen flex-col">
      <EditorProvider value={editorContext}>
        <div
          className={`flex flex-1 flex-col ${spotlightUser ? "border" : ""}`}
          style={{ borderColor: spotlightUser?.color || "" }}>
          <OverlayUI
            nodePickerOpen={nodePickerOpen}
            selectedNodeIds={selectedNodeIds}
            nodes={nodes}
            edges={edges}
            project={currentProject}
            yDoc={yDoc}
            self={self}
            users={users}
            spotlightUserClientId={spotlightUserClientId}
            isSaving={isSaving}
            allowedToDeploy={allowedToDeploy}
            canUndo={canUndo}
            canRedo={canRedo}
            isMainWorkflow={isMainWorkflow}
            rawWorkflows={rawWorkflows}
            openWorkflows={openWorkflows}
            currentWorkflowId={currentWorkflowId}
            customDebugRunWorkflowVariables={customDebugRunWorkflowVariables}
            openNodePickerViaShortcut={openNodePickerViaShortcut}
            onWorkflowChange={handleWorkflowChange}
            onWorkflowOpen={handleWorkflowOpen}
            onWorkflowClose={handleWorkflowClose}
            onNodesAdd={handleNodesAdd}
            onNodesChange={handleNodesChange}
            onNodePickerClose={handleNodePickerClose}
            onEdgesAdd={handleEdgesAdd}
            onEdgesChange={handleEdgesChange}
            onWorkflowRedo={handleWorkflowRedo}
            onWorkflowUndo={handleWorkflowUndo}
            onProjectShare={handleProjectShare}
            onProjectExport={handleCurrentProjectExport}
            onWorkflowDeployment={handleWorkflowDeployment}
            onDebugRunStart={handleDebugRunStart}
            onDebugRunStartFromSelectedNode={
              handleFromSelectedNodeDebugRunStart
            }
            onDebugRunStop={handleDebugRunStop}
            onDebugRunVariableValueChange={handleDebugRunVariableValueChange}
            onProjectSnapshotSave={handleProjectSnapshotSave}
            onSpotlightUserSelect={handleSpotlightUserSelect}
            onSpotlightUserDeselect={handleSpotlightUserDeselect}
            onLayoutChange={handleLayoutChange}
            onDebugRunJoin={loadExternalDebugJob}
            activeUsersDebugRuns={activeUsersDebugRuns}
            showSearchPanel={showSearchPanel}
            onShowSearchPanel={setShowSearchPanel}>
            <Canvas
              nodes={nodes}
              edges={edges}
              yDoc={yDoc}
              users={users}
              currentWorkflowId={currentWorkflowId}
              isMainWorkflow={isMainWorkflow}
              onWorkflowAdd={handleWorkflowAdd}
              onWorkflowOpen={handleWorkflowOpen}
              onWorkflowAddFromSelection={handleWorkflowAddFromSelection}
              onNodesAdd={handleNodesAdd}
              onBeforeDelete={handleBeforeDeleteNodes}
              onNodesChange={handleNodesChange}
              onNodeSettings={handleNodeSettings}
              onNodePickerOpen={handleNodePickerOpen}
              onEdgesAdd={handleEdgesAdd}
              onEdgesChange={handleEdgesChange}
              onCopy={handleCopy}
              onCut={handleCut}
              onPaste={handlePaste}
              onPaneMouseMove={handlePaneMouseMove}
              onNodesDisable={handleNodesDisable}
              onPaneClick={handlePaneClick}
              onDebugRunStartFromSelectedNode={
                handleFromSelectedNodeDebugRunStart
              }
            />
          </OverlayUI>

          {openNode && (
            <ParamsDialog
              openNode={openNode}
              onOpenNode={handleOpenNode}
              onDataSubmit={handleNodesDataUpdate}
              onWorkflowRename={handleWorkflowRename}
            />
          )}
          {showBeforeDeleteDialog && (
            <NodeDeletionDialog
              showBeforeDeleteDialog={showBeforeDeleteDialog}
              deferredDeleteRef={deferredDeleteRef}
              onDialogClose={handleDeleteDialogClose}
            />
          )}
        </div>
      </EditorProvider>
    </div>
  );
}
