import { type XYPosition } from "@xyflow/react";
import { memo, useCallback, useState } from "react";

import type {
  ActionNodeType,
  Algorithm,
  Direction,
  Edge,
  Node,
} from "@flow/types";

import {
  ActionBar,
  CanvasActionBar,
  Toolbox,
  Breadcrumb,
  Infobar,
  NodePickerDialog,
  LayoutOptionsDialog,
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
  isMainWorkflow: boolean;
  hasReader?: boolean;
  onWorkflowDeployment: (
    description: string,
    deploymentId?: string,
  ) => Promise<void>;
  onProjectShare: (share: boolean) => void;
  onNodesAdd: (nodes: Node[]) => void;
  onNodePickerClose: () => void;
  onRightPanelOpen: (content?: "version-history") => void;
  onWorkflowUndo: () => void;
  onWorkflowRedo: () => void;
  onLayoutChange: (
    algorithm: Algorithm,
    direction: Direction,
    spacing: number,
  ) => void;
  children?: React.ReactNode;
};

const OverlayUI: React.FC<OverlayUIProps> = ({
  hoveredDetails,
  nodePickerOpen,
  allowedToDeploy,
  canUndo,
  canRedo,
  isMainWorkflow,
  hasReader,
  onWorkflowDeployment,
  onProjectShare,
  onNodesAdd,
  onNodePickerClose,
  onRightPanelOpen,
  onWorkflowUndo,
  onWorkflowRedo,
  onLayoutChange,
  children: canvas,
}) => {
  const [showLayoutOptions, setShowLayoutOptions] = useState(false);

  const handleLayoutOptionsToggle = useCallback(() => {
    setShowLayoutOptions((prev) => !prev);
  }, []);

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
            isMainWorkflow={isMainWorkflow}
            hasReader={hasReader}
            onLayoutChange={handleLayoutOptionsToggle}
            onRedo={onWorkflowRedo}
            onUndo={onWorkflowUndo}
          />
        </div>
        <div id="right-top" className="absolute right-1 top-1 m-1">
          <ActionBar
            allowedToDeploy={allowedToDeploy}
            onProjectShare={onProjectShare}
            onWorkflowDeployment={onWorkflowDeployment}
            onRightPanelOpen={onRightPanelOpen}
          />
        </div>
        <div className="absolute bottom-2 right-2">
          <CanvasActionBar />
        </div>
        {hoveredDetails && <Infobar hoveredDetails={hoveredDetails} />}
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
