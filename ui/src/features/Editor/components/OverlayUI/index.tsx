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
  CanvasActionBar,
  Toolbox,
  Infobar,
  NodePickerDialog,
  LayoutOptionsDialog,
  DebugLogs,
  DebugPreview,
  JobStatus,
} from "./components";

type OverlayUIProps = {
  hoveredDetails: Node | Edge | undefined;
  nodePickerOpen?: {
    position: XYPosition;
    nodeType: ActionNodeType;
  };
  canUndo: boolean;
  canRedo: boolean;
  isMainWorkflow: boolean;
  hasReader?: boolean;
  onNodesAdd: (nodes: Node[]) => void;
  onNodePickerClose: () => void;
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
  canUndo,
  canRedo,
  isMainWorkflow,
  hasReader,
  onNodesAdd,
  onNodePickerClose,
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
          className="pointer-events-none absolute inset-x-0 top-0 flex shrink-0 justify-center *:pointer-events-auto"
        />
        <div
          id="left-top"
          className="pointer-events-none absolute bottom-1 left-2 top-2 flex flex-col shrink-0 gap-4 *:pointer-events-auto">
          <div className="ml-2 self-start">
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
        </div>
        <div id="right-top" className="absolute right-4 top-4">
          <JobStatus />
        </div>
        <div className="pointer-events-none absolute inset-y-2 left-2 flex items-end">
          <DebugLogs />
        </div>
        <div className="pointer-events-none absolute bottom-4 right-4 flex flex-row-reverse items-end gap-2">
          <CanvasActionBar />
          <DebugPreview />
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
