import { type XYPosition } from "@xyflow/react";
import { memo } from "react";

import type { ActionNodeType, Edge, Node } from "@flow/types";

import {
  ActionBar,
  CanvasActionBar,
  Toolbox,
  Breadcrumb,
  Infobar,
  NodePickerDialog,
} from "./components";

type OverlayUIProps = {
  hoveredDetails: Node | Edge | undefined;
  nodePickerOpen?: {
    position: XYPosition;
    nodeType: ActionNodeType;
  };
  allowedToDeploy: boolean;
  canUndo: boolean;
  canRedo: boolean;
  onWorkflowDeployment: (
    description: string,
    deploymentId?: string,
  ) => Promise<void>;
  onNodesAdd: (nodes: Node[]) => void;
  onNodePickerClose: () => void;
  onRightPanelOpen: (content?: "version-history") => void;
  onWorkflowUndo: () => void;
  onWorkflowRedo: () => void;
  isMainWorkflow: boolean;
  children?: React.ReactNode;
  hasReader?: boolean;
};

const OverlayUI: React.FC<OverlayUIProps> = ({
  hoveredDetails,
  nodePickerOpen,
  allowedToDeploy,
  canUndo,
  canRedo,
  onWorkflowDeployment,
  onNodesAdd,
  onNodePickerClose,
  onRightPanelOpen,
  onWorkflowUndo,
  onWorkflowRedo,
  isMainWorkflow,
  hasReader,
  children: canvas,
}) => {
  return (
    <>
      <div className="relative flex flex-1 flex-col">
        {/* {devMode && <DevTools />} */}
        {canvas}
        <div
          id="top-middle"
          className="pointer-events-none absolute inset-x-0 top-0 flex shrink-0 justify-center [&>*]:pointer-events-auto">
          <Breadcrumb />
        </div>
        <div
          id="left-top"
          className="pointer-events-none absolute bottom-1 left-2 top-2 flex shrink-0 gap-2 [&>*]:pointer-events-auto">
          <Toolbox
            canUndo={canUndo}
            canRedo={canRedo}
            onRedo={onWorkflowRedo}
            onUndo={onWorkflowUndo}
            isMainWorkflow={isMainWorkflow}
            hasReader={hasReader}
          />
        </div>
        <div id="right-top" className="absolute right-1 top-1 m-1">
          <ActionBar
            allowedToDeploy={allowedToDeploy}
            onWorkflowDeployment={onWorkflowDeployment}
            onRightPanelOpen={onRightPanelOpen}
          />
        </div>
        <div className="absolute bottom-2 right-2">
          <CanvasActionBar />
        </div>
        {hoveredDetails && <Infobar hoveredDetails={hoveredDetails} />}
      </div>
      {nodePickerOpen && (
        <NodePickerDialog
          openedActionType={nodePickerOpen}
          onNodesAdd={onNodesAdd}
          onClose={onNodePickerClose}
          isMainWorkflow={isMainWorkflow}
        />
      )}
    </>
  );
};

export default memo(OverlayUI);
