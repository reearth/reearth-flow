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
  JobStatus,
  DebugPanel,
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
          className="pointer-events-none absolute inset-x-0 top-2 flex shrink-0 justify-center *:pointer-events-auto">
          <JobStatus />
        </div>
        <div
          id="left-top"
          className="pointer-events-none absolute top-4 bottom-1 left-4 flex shrink-0 flex-col gap-4 *:pointer-events-auto">
          <div className="self-start">
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
        <div id="right-top" className="absolute top-4 right-4" />
        <div className="pointer-events-none absolute inset-y-2 bottom-4 left-4 flex items-end">
          <DebugPanel />
        </div>
        <div className="pointer-events-none absolute right-4 bottom-4 flex flex-row-reverse items-end gap-4">
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
