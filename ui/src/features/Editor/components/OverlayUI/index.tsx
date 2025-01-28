import { type XYPosition } from "@xyflow/react";
import { memo } from "react";

import type { ActionNodeType, Edge, Node } from "@flow/types";

import {
  ActionBar,
  CanvasActionBar,
  Toolbox,
  Breadcrumb,
  Infobar,
} from "./components";
import NodePickerDialog from "./components/NodePickerDialog";

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
    deploymentId?: string,
    description?: string,
  ) => Promise<void>;
  onNodesAdd: (nodes: Node[]) => void;
  onNodePickerClose: () => void;
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
        <Breadcrumb />
        <Toolbox
          canUndo={canUndo}
          canRedo={canRedo}
          onRedo={onWorkflowRedo}
          onUndo={onWorkflowUndo}
          isMainWorkflow={isMainWorkflow}
          hasReader={hasReader}
        />
        <ActionBar
          allowedToDeploy={allowedToDeploy}
          onWorkflowDeployment={onWorkflowDeployment}
        />
        <CanvasActionBar />
        <Infobar hoveredDetails={hoveredDetails} />
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
