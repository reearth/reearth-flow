import { Edge, EdgeChange, NodeChange, type XYPosition } from "@xyflow/react";
import { memo, useCallback } from "react";
import { Doc } from "yjs";

import { useEditorContext } from "@flow/features/Editor/editorContext";
import type {
  ActionNodeType,
  Algorithm,
  AnyWorkflowVariable,
  AwarenessUser,
  Direction,
  Node,
  Project,
  Workflow,
} from "@flow/types";

import {
  ActionBar,
  DebugActionBar,
  CanvasActionBar,
  Toolbox,
  ActionPickerDialog,
  LayoutSubToolbar,
  DebugPanel,
  Homebar,
  VersionDialog,
  SearchActionBar,
  LockedBadge,
} from "./components";
import useHooks from "./hooks";

type OverlayUIProps = {
  nodePickerOpen?: {
    position: XYPosition;
    nodeType: ActionNodeType;
  };
  selectedNodeIds: string[];
  nodes: Node[];
  edges?: Edge[];
  canUndo: boolean;
  canRedo: boolean;
  isMainWorkflow: boolean;
  rawWorkflows: Workflow[];
  project?: Project;
  yDoc: Doc | null;
  openWorkflows: {
    id: string;
    name: string;
  }[];
  currentWorkflowId: string;
  customDebugRunWorkflowVariables?: AnyWorkflowVariable[];
  workflowVariableDefaults?: AnyWorkflowVariable[];
  openNodePickerViaShortcut: boolean;
  refetchWorkflowVariables: () => void;
  onNodesAdd: (nodes: Node[]) => void;
  onNodesChange?: (changes: NodeChange<Node>[]) => void;
  onWorkflowAdd?: (position?: XYPosition) => Promise<void>;
  onNodePickerOpen?: (
    position: XYPosition,
    nodeType?: ActionNodeType,
    isMainWorkflow?: boolean,
  ) => void;
  onNodePickerClose: () => void;
  onEdgesAdd?: (edges: Edge[]) => void;
  onEdgesChange?: (changes: EdgeChange[]) => void;
  onWorkflowUndo: () => void;
  onWorkflowRedo: () => void;
  onLayoutChange: (
    algorithm: Algorithm,
    direction: Direction,
    applyToAll: boolean,
  ) => void;
  self: AwarenessUser;
  users: Record<string, AwarenessUser>;
  spotlightUserClientId: number | null;
  allowedToDeploy: boolean;
  isSaving: boolean;
  onWorkflowClose: (workflowId: string) => void;
  onWorkflowOpen: (workflowId: string) => void;
  onWorkflowChange: (workflowId?: string) => void;
  onWorkflowDeployment: (
    description: string,
    deploymentId?: string,
  ) => Promise<void>;
  onProjectExport: () => void;
  sharingUrl?: string;
  onProjectShare: (share: boolean) => void;
  onDebugRunStart: () => Promise<void>;
  onDebugRunStartFromSelectedNode?: (
    node?: Node,
    nodes?: Node[],
  ) => Promise<void>;
  onDebugRunStop: () => Promise<void>;
  onResetDebugRunWorkflowVariables: () => void;
  onDebugRunJoin?: (jobId: string, userName: string) => Promise<void>;
  onProjectSnapshotSave: () => Promise<void>;
  onProjectLockChange: (lock: boolean) => void;
  onSpotlightUserSelect: (clientId: number) => void;
  onSpotlightUserDeselect: () => void;
  activeUsersDebugRuns?: AwarenessUser[];
  children?: React.ReactNode;
  showSearchPanel: boolean;
  onShowSearchPanel: (open: boolean) => void;
  onUserFocusedElement?: (isOpen: boolean) => void;
};

const OverlayUI: React.FC<OverlayUIProps> = ({
  nodePickerOpen,
  selectedNodeIds,
  nodes,
  edges,
  canUndo,
  canRedo,
  isMainWorkflow,
  rawWorkflows,
  yDoc,
  project,
  allowedToDeploy,
  isSaving,
  self,
  users,
  spotlightUserClientId,
  openWorkflows,
  currentWorkflowId,
  customDebugRunWorkflowVariables,
  workflowVariableDefaults,
  openNodePickerViaShortcut,
  refetchWorkflowVariables,
  onNodesAdd,
  onNodesChange,
  onWorkflowAdd,
  onNodePickerOpen,
  onNodePickerClose,
  onEdgesAdd,
  onEdgesChange,
  onWorkflowUndo,
  onWorkflowRedo,
  onWorkflowChange,
  onWorkflowOpen,
  onWorkflowClose,
  onLayoutChange,
  onWorkflowDeployment,
  sharingUrl,
  onProjectExport,
  onProjectShare,
  onDebugRunStart,
  onDebugRunStartFromSelectedNode,
  onDebugRunStop,
  onResetDebugRunWorkflowVariables,
  onDebugRunJoin,
  onProjectSnapshotSave,
  onProjectLockChange,
  onSpotlightUserSelect,
  onSpotlightUserDeselect,
  children: canvas,
  activeUsersDebugRuns,
  showSearchPanel,
  onShowSearchPanel,
  onUserFocusedElement,
}) => {
  const { isLocked } = useEditorContext();
  const { showDialog, handleDialogOpen, handleDialogClose } = useHooks({
    onUserFocusedElement,
  });

  const handleLayoutOptionsToggle = useCallback(() => {
    if (showDialog === "layout") {
      handleDialogClose();
      return;
    } else {
      handleDialogOpen("layout");
    }
  }, [showDialog, handleDialogOpen, handleDialogClose]);

  return (
    <>
      <div
        className={`relative flex flex-1 flex-col border ${isMainWorkflow ? "border-transparent" : "border-node-subworkflow"}`}>
        {/* {devMode && <DevTools />} */}
        {canvas}
        <div
          id="top-middle"
          className="pointer-events-none absolute inset-x-0 top-2 flex shrink-0 justify-center">
          <div className="flex flex-col items-center gap-2">
            <div className="pointer-events-auto">
              <Toolbox
                canUndo={canUndo}
                canRedo={canRedo}
                isMainWorkflow={isMainWorkflow}
                showLayoutOptions={showDialog === "layout"}
                onLayoutChange={handleLayoutOptionsToggle}
                onNodesAdd={onNodesAdd}
                onWorkflowAdd={onWorkflowAdd}
                onNodePickerOpen={onNodePickerOpen}
                onRedo={onWorkflowRedo}
                onUndo={onWorkflowUndo}
              />
            </div>
            {showDialog === "layout" && !isLocked && (
              <div className="pointer-events-auto z-10">
                <LayoutSubToolbar
                  Ydoc={yDoc}
                  onLayoutChange={onLayoutChange}
                  onClose={handleDialogClose}
                />
              </div>
            )}
            {isLocked && (
              <div className="pointer-events-auto z-10">
                <LockedBadge onUnlock={() => onProjectLockChange(false)} />
              </div>
            )}
          </div>
        </div>
        <div
          id="left-top"
          className="pointer-events-none absolute top-2 left-2 *:pointer-events-auto">
          <Homebar
            isMainWorkflow={isMainWorkflow}
            self={self}
            users={users}
            spotlightUserClientId={spotlightUserClientId}
            currentWorkflowId={currentWorkflowId}
            openWorkflows={openWorkflows}
            onWorkflowChange={onWorkflowChange}
            onWorkflowClose={onWorkflowClose}
            onSpotlightUserSelect={onSpotlightUserSelect}
            onSpotlightUserDeselect={onSpotlightUserDeselect}
            onUserFocusedElement={onUserFocusedElement}
          />
        </div>
        <div id="right-top" className="absolute top-2 right-2 h-[42px]">
          <div
            className={`flex h-full items-center justify-center gap-2 self-center rounded-xl border border-border bg-secondary/70 p-1 shadow-md shadow-[black]/10 backdrop-blur-xs select-none dark:border-primary dark:shadow-secondary ${!isMainWorkflow ? "border-node-subworkflow" : ""}`}>
            <DebugActionBar
              activeUsersDebugRuns={activeUsersDebugRuns}
              selectedNodeIds={selectedNodeIds}
              edges={edges}
              isSaving={isSaving}
              onDebugRunJoin={onDebugRunJoin}
              onDebugRunStart={onDebugRunStart}
              onDebugRunStartFromSelectedNode={onDebugRunStartFromSelectedNode}
              onDebugRunStop={onDebugRunStop}
              onResetDebugRunWorkflowVariables={
                onResetDebugRunWorkflowVariables
              }
              customDebugRunWorkflowVariables={customDebugRunWorkflowVariables}
              workflowVariableDefaults={workflowVariableDefaults}
              onUserFocusedElement={onUserFocusedElement}
              refetchWorkflowVariables={refetchWorkflowVariables}
            />
            <div className="h-4/5 border-r" />
            <ActionBar
              allowedToDeploy={allowedToDeploy}
              isSaving={isSaving}
              showDialog={showDialog}
              onDialogOpen={handleDialogOpen}
              onDialogClose={handleDialogClose}
              sharingUrl={sharingUrl}
              onProjectShare={onProjectShare}
              onProjectExport={onProjectExport}
              onWorkflowDeployment={onWorkflowDeployment}
              onProjectLockChange={onProjectLockChange}
              onProjectSnapshotSave={onProjectSnapshotSave}
            />
          </div>
        </div>
        {showDialog === "version" && (
          <VersionDialog
            project={project}
            yDoc={yDoc}
            onDialogClose={handleDialogClose}
          />
        )}
        <div
          id="left-bottom-search-bar"
          className="pointer-events-none absolute bottom-2 left-2">
          <div className="pointer-events-auto">
            <SearchActionBar
              rawWorkflows={rawWorkflows}
              currentWorkflowId={currentWorkflowId}
              onWorkflowOpen={onWorkflowOpen}
              onNodesChange={onNodesChange}
              showSearchPanel={showSearchPanel}
              onShowSearchPanel={onShowSearchPanel}
            />
          </div>
        </div>
        <div id="middle-bottom-debug-panel">
          <DebugPanel />
        </div>
        <div
          id="right-bottom-canvas-action-bar"
          className="pointer-events-none absolute right-2 bottom-2 z-10">
          <div className="pointer-events-auto">
            <CanvasActionBar />
          </div>
        </div>
      </div>
      {nodePickerOpen && (
        <ActionPickerDialog
          openedActionType={nodePickerOpen}
          isMainWorkflow={isMainWorkflow}
          nodes={nodes}
          selectedNodeIds={selectedNodeIds}
          edges={edges}
          openNodePickerViaShortcut={openNodePickerViaShortcut}
          onNodesAdd={onNodesAdd}
          onEdgesAdd={onEdgesAdd}
          onEdgesChange={onEdgesChange}
          onClose={onNodePickerClose}
        />
      )}
    </>
  );
};

export default memo(OverlayUI);
