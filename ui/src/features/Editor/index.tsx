import { useMemo } from "react";
import type { Awareness } from "y-protocols/awareness";
import { Doc, Map as YMap, UndoManager as YUndoManager } from "yjs";

import Canvas from "@flow/features/Canvas";
import { YWorkflow } from "@flow/lib/yjs/types";

import { OverlayUI, ParamsDialog, NodeDeletionDialog } from "./components";
import { EditorContextType, EditorProvider } from "./editorContext";
import useHooks from "./hooks";
import PreviewSchemaMonitors from "./usePreviewSchema/PreviewSchemaMonitors";

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
    spotlightFollow,
    userActivities,
    activeUsersDebugRuns,
    rawWorkflows,
    customDebugRunWorkflowVariables,
    refetchWorkflowVariables,
    showSearchPanel,
    openNodePickerViaShortcut,
    workflowVariableDefaults,
    loadExternalDebugJob,
    handleWorkflowAdd,
    handleWorkflowDeployment,
    sharingUrl,
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
    handleResetDebugRunWorkflowVariables,
    schemaProbes,
    readerAttributeSuggestions,
    handleNodeParamsSaved,
    handleProbeComplete,
    handleProbeError,
    handleLayoutChange,
    handleCopy,
    handleCut,
    handlePaste,
    handleProjectSnapshotSave,
    isLocked,
    isReaderRestricted,
    handleProjectLockChange,
    handleSpotlightUserSelect,
    handleSpotlightUserDeselect,
    handleNodesDisable,
    handlePaneClick,
    handlePointerDown,
    handleConnectStart,
    handleConnectEnd,
    handleParamFieldFocus,
    handleWorkflowVarDialogOpen,
    handleWorkflowVarDialogClose,
    handleWorkflowVarFieldFocus,
    handleWorkflowVarEditStart,
    handleUserFocusedElement,
    handleDialogAwareness,
    handleSubEditorAwareness,
    handleVersionSnapshotAwareness,
    awarenessSelectionsMap,
    awarenessEchoMap,
    setEchoElement,
    handleShowSearchPanel,
    selectedNodeIds,
    staleNodeIds,
  } = useHooks({
    yDoc,
    yWorkflows,
    yAwareness,
    undoManager,
    undoTrackerActionWrapper,
  });

  const editorContext = useMemo(
    (): EditorContextType => ({
      isLocked,
      isReaderRestricted,
      canViewIntermediateData: true,
      onNodesChange: handleNodesChange,
      onNodeSettings: handleNodeSettings,
      currentYWorkflow,
      undoTrackerActionWrapper,
      awarenessSelectionsMap,
      awarenessEchoMap,
      onEchoElement: setEchoElement,
      spotlightFollow,
      yDoc,
      workflowVarAwareness: {
        onDialogOpen: handleWorkflowVarDialogOpen,
        onDialogClose: handleWorkflowVarDialogClose,
        onFieldFocus: handleWorkflowVarFieldFocus,
        onEditStart: handleWorkflowVarEditStart,
      },
      staleNodeIds,
    }),
    [
      isLocked,
      isReaderRestricted,
      handleNodesChange,
      handleNodeSettings,
      currentYWorkflow,
      undoTrackerActionWrapper,
      awarenessSelectionsMap,
      awarenessEchoMap,
      setEchoElement,
      spotlightFollow,
      yDoc,
      handleWorkflowVarDialogOpen,
      handleWorkflowVarDialogClose,
      handleWorkflowVarFieldFocus,
      handleWorkflowVarEditStart,
      staleNodeIds,
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
            spotlightFollow={spotlightFollow}
            userActivities={userActivities}
            onDialogAwareness={handleDialogAwareness}
            onVersionSnapshotAwareness={handleVersionSnapshotAwareness}
            isSaving={isSaving}
            allowedToDeploy={allowedToDeploy}
            canUndo={canUndo}
            canRedo={canRedo}
            isMainWorkflow={isMainWorkflow}
            rawWorkflows={rawWorkflows}
            openWorkflows={openWorkflows}
            currentWorkflowId={currentWorkflowId}
            customDebugRunWorkflowVariables={customDebugRunWorkflowVariables}
            workflowVariableDefaults={workflowVariableDefaults}
            openNodePickerViaShortcut={openNodePickerViaShortcut}
            refetchWorkflowVariables={refetchWorkflowVariables}
            onWorkflowChange={handleWorkflowChange}
            onWorkflowOpen={handleWorkflowOpen}
            onWorkflowClose={handleWorkflowClose}
            onNodesAdd={handleNodesAdd}
            onNodesChange={handleNodesChange}
            onWorkflowAdd={handleWorkflowAdd}
            onNodePickerOpen={handleNodePickerOpen}
            onNodePickerClose={handleNodePickerClose}
            onEdgesAdd={handleEdgesAdd}
            onEdgesChange={handleEdgesChange}
            onWorkflowRedo={handleWorkflowRedo}
            onWorkflowUndo={handleWorkflowUndo}
            sharingUrl={sharingUrl}
            onProjectShare={handleProjectShare}
            onProjectExport={handleCurrentProjectExport}
            onWorkflowDeployment={handleWorkflowDeployment}
            onDebugRunStart={handleDebugRunStart}
            onDebugRunStartFromSelectedNode={
              handleFromSelectedNodeDebugRunStart
            }
            onDebugRunStop={handleDebugRunStop}
            onResetDebugRunWorkflowVariables={
              handleResetDebugRunWorkflowVariables
            }
            onProjectSnapshotSave={handleProjectSnapshotSave}
            onProjectLockChange={handleProjectLockChange}
            onSpotlightUserSelect={handleSpotlightUserSelect}
            onSpotlightUserDeselect={handleSpotlightUserDeselect}
            onLayoutChange={handleLayoutChange}
            onDebugRunJoin={loadExternalDebugJob}
            activeUsersDebugRuns={activeUsersDebugRuns}
            showSearchPanel={showSearchPanel}
            onShowSearchPanel={handleShowSearchPanel}
            onUserFocusedElement={handleUserFocusedElement}>
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
              onNodesDisable={handleNodesDisable}
              onPaneClick={handlePaneClick}
              onDebugRunStartFromSelectedNode={
                handleFromSelectedNodeDebugRunStart
              }
              onConnectStart={handleConnectStart}
              onConnectEnd={handleConnectEnd}
              onPointerDown={handlePointerDown}
            />
          </OverlayUI>

          {openNode && (
            <ParamsDialog
              yDoc={yDoc}
              users={users}
              openNode={openNode}
              onOpenNode={handleOpenNode}
              onDataSubmit={handleNodesDataUpdate}
              onNodeParamsSaved={handleNodeParamsSaved}
              attributeSuggestions={readerAttributeSuggestions}
              onWorkflowRename={handleWorkflowRename}
              onParamFieldFocus={handleParamFieldFocus}
              onSubEditorAwareness={handleSubEditorAwareness}
              isFollowing={spotlightFollow.isFollowing}
              followSubEditor={spotlightFollow.subEditor}
              onSpotlightUserDeselect={handleSpotlightUserDeselect}
            />
          )}
          <PreviewSchemaMonitors
            probes={schemaProbes}
            onComplete={handleProbeComplete}
            onError={handleProbeError}
          />
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
