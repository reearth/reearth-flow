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
  nodes: Node[];
  onWorkflowDeployment: (
    deploymentId?: string,
    description?: string,
  ) => Promise<void>;
  onNodesChange: (nodes: Node[]) => void;
  onNodeLocking: (nodeId: string) => void;
  onNodePickerClose: () => void;
  onWorkflowUndo: () => void;
  onWorkflowRedo: () => void;
  children?: React.ReactNode;
};

const OverlayUI: React.FC<OverlayUIProps> = ({
  hoveredDetails,
  nodePickerOpen,
  nodes,
  onWorkflowDeployment,
  onNodesChange,
  onNodeLocking,
  onNodePickerClose,
  onWorkflowUndo,
  onWorkflowRedo,
  children: canvas,
}) => {
  // const { devMode } = config();
  return (
    <>
      <div className="relative flex flex-1 flex-col">
        {/* {devMode && <DevTools />} */}
        {canvas}
        <Breadcrumb />
        <Toolbox onRedo={onWorkflowRedo} onUndo={onWorkflowUndo} />
        <ActionBar onWorkflowDeployment={onWorkflowDeployment} />
        <CanvasActionBar />
        <Infobar hoveredDetails={hoveredDetails} />
      </div>
      {nodePickerOpen && (
        <NodePickerDialog
          openedActionType={nodePickerOpen}
          nodes={nodes}
          onNodesChange={onNodesChange}
          onNodeLocking={onNodeLocking}
          onClose={onNodePickerClose}
        />
      )}
    </>
  );
};

export default memo(OverlayUI);
