import { Array as YArray } from "yjs";

import { Button } from "@flow/components";
import Canvas from "@flow/features/Canvas";
import { useT } from "@flow/lib/i18n";
import { YWorkflow } from "@flow/lib/yjs/types";
import { Project } from "@flow/types";

import { ParamsPanel, WorkflowTabs } from "../Editor/components";

import useHooks from "./hooks";

type Props = {
  yWorkflows: YArray<YWorkflow>;
  project?: Project;
  undoTrackerActionWrapper: (callback: () => void) => void;
};

const SharedCanvas: React.FC<Props> = ({
  yWorkflows,
  project,
  undoTrackerActionWrapper,
}) => {
  const t = useT();
  const {
    currentWorkflowId,
    nodes,
    edges,
    openWorkflows,
    locallyLockedNode,
    // isMainWorkflow,
    // hoveredDetails,
    handleProjectExport,
    // handleNodeHover,
    // handleEdgeHover,
    handleNodeDoubleClick,
    // handleWorkflowOpen,
    handleNodesChange,
    handleWorkflowClose,
    handleCurrentWorkflowIdChange,
  } = useHooks({ yWorkflows, project, undoTrackerActionWrapper });
  return (
    <div className="relative flex size-full flex-col">
      <Canvas
        nodes={nodes}
        edges={edges}
        canvasLock
        // onNodeHover={handleNodeHover}
        onNodesChange={handleNodesChange}
        onNodeDoubleClick={handleNodeDoubleClick}
        // onEdgeHover={handleEdgeHover}
      />
      <WorkflowTabs
        className="max-w-full bg-secondary px-1"
        openWorkflows={openWorkflows}
        currentWorkflowId={currentWorkflowId}
        onWorkflowClose={handleWorkflowClose}
        onWorkflowChange={handleCurrentWorkflowIdChange}
      />
      <div className="absolute right-0 top-0 p-4">
        <Button size="lg" onClick={handleProjectExport}>
          {t("Export Project")}
        </Button>
      </div>
      <ParamsPanel selected={locallyLockedNode} />
    </div>
  );
};

export default SharedCanvas;
