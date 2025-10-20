import { type XYPosition } from "@xyflow/react";
import { memo, useCallback, useState } from "react";
import { Doc } from "yjs";

import type {
  ActionNodeType,
  Algorithm,
  AwarenessUser,
  Direction,
  Node,
  Project,
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
} from "./components";
import useHooks from "./hooks";

type OverlayUIProps = {
  nodePickerOpen?: {
    position: XYPosition;
    nodeType: ActionNodeType;
  };
  canUndo: boolean;
  canRedo: boolean;
  isMainWorkflow: boolean;
  project?: Project;
  yDoc: Doc | null;
  openWorkflows: {
    id: string;
    name: string;
  }[];
  currentWorkflowId: string;
  onNodesAdd: (nodes: Node[]) => void;
  onNodePickerClose: () => void;
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
  onWorkflowChange: (workflowId?: string) => void;
  onWorkflowDeployment: (
    description: string,
    deploymentId?: string,
  ) => Promise<void>;
  onProjectExport: () => void;
  onProjectShare: (share: boolean) => void;
  onDebugRunStart: () => Promise<void>;
  onDebugRunStop: () => Promise<void>;
  onProjectSnapshotSave: () => Promise<void>;
  onSpotlightUserSelect: (clientId: number) => void;
  onSpotlightUserDeselect: () => void;
  children?: React.ReactNode;
};

const OverlayUI: React.FC<OverlayUIProps> = ({
  nodePickerOpen,
  canUndo,
  canRedo,
  isMainWorkflow,
  yDoc,
  project,
  allowedToDeploy,
  isSaving,
  self,
  users,
  spotlightUserClientId,
  openWorkflows,
  currentWorkflowId,
  onNodesAdd,
  onNodePickerClose,
  onWorkflowUndo,
  onWorkflowRedo,
  onWorkflowChange,
  onWorkflowClose,
  onLayoutChange,
  onWorkflowDeployment,
  onProjectExport,
  onProjectShare,
  onDebugRunStart,
  onDebugRunStop,
  onProjectSnapshotSave,
  onSpotlightUserSelect,
  onSpotlightUserDeselect,
  children: canvas,
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
          {isMainWorkflow && (
            <div
              className={`flex h-full items-center justify-center gap-2 self-center rounded-xl border border-border bg-secondary/70 p-1 shadow-md shadow-[black]/20 backdrop-blur-xs select-none dark:border-primary dark:shadow-secondary ${!isMainWorkflow ? "border-node-subworkflow" : ""}`}>
              <DebugActionBar
                onDebugRunStart={onDebugRunStart}
                onDebugRunStop={onDebugRunStop}
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
          )}
        </div>
        {showDialog === "version" && (
          <VersionDialog
            project={project}
            yDoc={yDoc}
            onDialogClose={handleDialogClose}
          />
        )}
        <div id="left-bottom" className="absolute bottom-2 left-2 z-1">
          <DebugPanel />
        </div>
        <div
          id="bottom-middle"
          className="pointer-events-none absolute inset-x-0 bottom-2 flex shrink-0 justify-center *:pointer-events-auto"
        />
        <div
          id="right-bottom"
          className="pointer-events-none absolute right-2 bottom-2 flex flex-row-reverse items-end gap-4">
          <CanvasActionBar />
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
          onNodesAdd={onNodesAdd}
          onClose={onNodePickerClose}
        />
      )}
    </>
  );
};

export default memo(OverlayUI);
