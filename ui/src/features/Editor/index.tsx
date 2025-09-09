import { useMemo } from "react";
import type { Awareness } from "y-protocols/awareness";
import { Doc, Map as YMap, UndoManager as YUndoManager } from "yjs";

import Canvas from "@flow/features/Canvas";
import { YWorkflow } from "@flow/lib/yjs/types";

import {
  TopBar,
  OverlayUI,
  ParamsDialog,
  NodeDeletionDialog,
} from "./components";
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
    selectedEdgeIds,
    openNode,
    nodePickerOpen,
    canUndo,
    canRedo,
    allowedToDeploy,
    isMainWorkflow,
    deferredDeleteRef,
    showBeforeDeleteDialog,
    isSaving,
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
    handleNodeDataUpdate,
    handleNodeSettings,
    handleOpenNode,
    handleNodePickerOpen,
    handleNodePickerClose,
    handleEdgesAdd,
    handleEdgesChange,
    handleWorkflowRedo,
    handleWorkflowUndo,
    handleWorkflowRename,
    handleDebugRunStart,
    handleDebugRunStop,
    handleLayoutChange,
    handleCopy,
    handleCut,
    handlePaste,
    handleProjectSnapshotSave,
    handlePaneMouseMove,
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
    }),
    [handleNodesChange, handleNodeSettings, currentYWorkflow],
  );

  return (
    <div className="flex h-screen flex-col">
      <EditorProvider value={editorContext}>
        <TopBar
          currentWorkflowId={currentWorkflowId}
          project={currentProject}
          yDoc={yDoc}
          openWorkflows={openWorkflows}
          allowedToDeploy={allowedToDeploy}
          isSaving={isSaving}
          self={self}
          users={users}
          onProjectShare={handleProjectShare}
          onProjectExport={handleCurrentProjectExport}
          onWorkflowDeployment={handleWorkflowDeployment}
          onWorkflowClose={handleWorkflowClose}
          onWorkflowChange={handleWorkflowChange}
          onDebugRunStart={handleDebugRunStart}
          onDebugRunStop={handleDebugRunStop}
          onProjectSnapshotSave={handleProjectSnapshotSave}
        />

        <div className="flex flex-1 flex-col">
          <OverlayUI
            nodePickerOpen={nodePickerOpen}
            canUndo={canUndo}
            canRedo={canRedo}
            isMainWorkflow={isMainWorkflow}
            onNodesAdd={handleNodesAdd}
            onNodePickerClose={handleNodePickerClose}
            onWorkflowUndo={handleWorkflowUndo}
            onWorkflowRedo={handleWorkflowRedo}
            onLayoutChange={handleLayoutChange}>
            <Canvas
              nodes={nodes}
              edges={edges}
              selectedEdgeIds={selectedEdgeIds}
              yDoc={yDoc}
              users={users}
              onWorkflowAdd={handleWorkflowAdd}
              onWorkflowOpen={handleWorkflowOpen}
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
            />
          </OverlayUI>

          {openNode && (
            <ParamsDialog
              openNode={openNode}
              onOpenNode={handleOpenNode}
              onDataSubmit={handleNodeDataUpdate}
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
