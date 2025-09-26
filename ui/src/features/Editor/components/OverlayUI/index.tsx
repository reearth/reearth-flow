import { type XYPosition } from "@xyflow/react";
import { memo, useCallback, useState } from "react";

import type { ActionNodeType, Algorithm, Direction, Node } from "@flow/types";

import {
  CanvasActionBar,
  Toolbox,
  NodePickerDialog,
  LayoutOptionsDialog,
  JobStatus,
  DebugPanel,
} from "./components";

type OverlayUIProps = {
  nodePickerOpen?: {
    position: XYPosition;
    nodeType: ActionNodeType;
  };
  canUndo: boolean;
  canRedo: boolean;
  isMainWorkflow: boolean;
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
  nodePickerOpen,
  canUndo,
  canRedo,
  isMainWorkflow,
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
          <Toolbox
            canUndo={canUndo}
            canRedo={canRedo}
            isMainWorkflow={isMainWorkflow}
            onLayoutChange={handleLayoutOptionsToggle}
            onRedo={onWorkflowRedo}
            onUndo={onWorkflowUndo}
          />
          <JobStatus />
        </div>
        <div
          id="left-top"
          className="pointer-events-none absolute top-2 bottom-1 left-2 flex shrink-0 flex-col gap-4 *:pointer-events-auto">
          <div className="self-start" />
        </div>
        <div id="right-top" className="absolute top-4 right-4" />
        <div id="left-bottom" className="absolute bottom-4 left-4">
          <DebugPanel />
        </div>
        <div
          id="bottom-middle"
          className="pointer-events-none absolute inset-x-0 bottom-4 flex shrink-0 justify-center *:pointer-events-auto"
        />
        <div
          id="right-bottom"
          className="pointer-events-none absolute right-4 bottom-4 flex flex-row-reverse items-end gap-4">
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
