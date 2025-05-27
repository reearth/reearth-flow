import { Map as YMap } from "yjs";

import Canvas from "@flow/features/Canvas";
import type { YWorkflow } from "@flow/lib/yjs/types";

import { ParamsPanel, WorkflowTabs } from "../Editor/components";

import useHooks from "./hooks";

type Props = {
  yWorkflows: YMap<YWorkflow>;
};

const VersionCanvas: React.FC<Props> = ({ yWorkflows }) => {
  const {
    currentWorkflowId,
    isSubworkflow,
    nodes,
    edges,
    openWorkflows,
    locallyLockedNode,
    handleWorkflowClose,
    handleCurrentWorkflowIdChange,
  } = useHooks({ yWorkflows });

  return (
    <div className="relative flex size-full flex-col">
      <Canvas
        isSubworkflow={isSubworkflow}
        nodes={nodes}
        edges={edges}
        canvasLock
      />
      <WorkflowTabs
        openWorkflows={openWorkflows}
        currentWorkflowId={currentWorkflowId}
        onWorkflowClose={handleWorkflowClose}
        onWorkflowChange={handleCurrentWorkflowIdChange}
      />
      <ParamsPanel selected={locallyLockedNode} />
    </div>
  );
};

export default VersionCanvas;
