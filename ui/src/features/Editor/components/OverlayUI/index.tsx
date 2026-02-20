import { Edge, EdgeChange, NodeChange, type XYPosition } from "@xyflow/react";
import { memo, useCallback, useState } from "react";
import { Doc } from "yjs";

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
  NodePickerDialog,
  LayoutOptionsDialog,
  DebugPanel,
  Homebar,
  VersionDialog,
  SearchActionBar,
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
  onNodesAdd: (nodes: Node[]) => void;
  onNodesChange?: (changes: NodeChange<Node>[]) => void;
  onNodePickerClose: () => void;
  onEdgesAdd?: (edges: Edge[]) => void;
  onEdgesChange?: (changes: EdgeChange[]) => void;
  onWorkflowUndo: () => void;
  onWorkflowRedo: () => void;
  onLayoutChange: (
    algorithm: Algorithm,
    direction: Direction,
    spacing: number,
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
  onProjectShare: (share: boolean) => void;
  onDebugRunStart: () => Promise<void>;
  onDebugRunStartFromSelectedNode?: (
    node?: Node,
    nodes?: Node[],
  ) => Promise<void>;
  onDebugRunStop: () => Promise<void>;
  onDebugRunVariableValueChange: (index: number, newValue: any) => void;
  onDebugRunJoin?: (jobId: string, userName: string) => Promise<void>;
  onProjectSnapshotSave: () => Promise<void>;
  onSpotlightUserSelect: (clientId: number) => void;
  onSpotlightUserDeselect: () => void;
  activeUsersDebugRuns?: AwarenessUser[];
  children?: React.ReactNode;
  showSearchPanel: boolean;
  onShowSearchPanel: (open: boolean) => void;
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
  onNodesAdd,
  onNodesChange,
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
  onProjectExport,
  onProjectShare,
  onDebugRunStart,
  onDebugRunStartFromSelectedNode,
  onDebugRunStop,
  onDebugRunVariableValueChange,
  onDebugRunJoin,
  onProjectSnapshotSave,
  onSpotlightUserSelect,
  onSpotlightUserDeselect,
  children: canvas,
  activeUsersDebugRuns,
  showSearchPanel,
  onShowSearchPanel,
}) => {
  const [showLayoutOptions, setShowLayoutOptions] = useState(false);
  const { showDialog, handleDialogOpen, handleDialogClose } = useHooks();

  const handleLayoutOptionsToggle = useCallback(() => {
    setShowLayoutOptions((prev) => !prev);
  }, []);

  return (
    <>
      <div
        className={`relative flex flex-1 flex-col border ${isMainWorkflow ? "border-transparent" : "border-node-subworkflow"}`}>
        {/* {devMode && <DevTools />} */}
        {canvas}
        <div
          id="top-middle"
          className="pointer-events-none absolute inset-x-0 top-2 flex shrink-0 justify-center *:pointer-events-auto">
          <Toolbox
            canUndo={canUndo}
            canRedo={canRedo}
            isMainWorkflow={isMainWorkflow}
            onLayoutChange={handleLayoutOptionsToggle}
            onRedo={onWorkflowRedo}
            onUndo={onWorkflowUndo}
          />
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
              customDebugRunWorkflowVariables={customDebugRunWorkflowVariables}
              onDebugRunVariableValueChange={onDebugRunVariableValueChange}
            />
            <div className="h-4/5 border-r" />
            <ActionBar
              allowedToDeploy={allowedToDeploy}
              isSaving={isSaving}
              showDialog={showDialog}
              onDialogOpen={handleDialogOpen}
              onDialogClose={handleDialogClose}
              onProjectShare={onProjectShare}
              onProjectExport={onProjectExport}
              onWorkflowDeployment={onWorkflowDeployment}
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
      <LayoutOptionsDialog
        isOpen={showLayoutOptions}
        onLayoutChange={onLayoutChange}
        onClose={handleLayoutOptionsToggle}
      />
      {nodePickerOpen && (
        <NodePickerDialog
          openedActionType={nodePickerOpen}
          isMainWorkflow={isMainWorkflow}
          nodes={nodes}
          edges={edges}
          onNodesAdd={onNodesAdd}
          onNodesChange={onNodesChange}
          onEdgesAdd={onEdgesAdd}
          onEdgesChange={onEdgesChange}
          onClose={onNodePickerClose}
        />
      )}
    </>
  );
};

export default memo(OverlayUI);
