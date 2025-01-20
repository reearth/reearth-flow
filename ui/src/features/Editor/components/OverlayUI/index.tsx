import { type XYPosition } from "@xyflow/react";
import { useMemo, memo } from "react";

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
  nodes: Node[];
  canUndo: boolean;
  canRedo: boolean;
  onWorkflowDeployment: (
    deploymentId?: string,
    description?: string,
  ) => Promise<void>;
  onNodesChange: (nodes: Node[]) => void;
  onNodePickerClose: () => void;
  onWorkflowUndo: () => void;
  onWorkflowRedo: () => void;
  isMainWorkflow: boolean;
  children?: React.ReactNode;
  hasReader: boolean;
};

const OverlayUI: React.FC<OverlayUIProps> = ({
  hoveredDetails,
  nodePickerOpen,
  nodes,
  canUndo,
  canRedo,
  onWorkflowDeployment,
  onNodesChange,
  onNodePickerClose,
  onWorkflowUndo,
  onWorkflowRedo,
  isMainWorkflow,
  hasReader,
  children: canvas,
}) => {
  // const { devMode } = config();
  const allowedToDeploy = useMemo(() => nodes.length > 0, [nodes]);
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
          nodes={nodes}
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
          nodes={nodes}
          onNodesChange={onNodesChange}
          onClose={onNodePickerClose}
          isMainWorkflow={isMainWorkflow}
        />
      )}
    </>
  );
};

export default memo(OverlayUI);
