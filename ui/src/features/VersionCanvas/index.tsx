import { Map as YMap } from "yjs";

import Canvas from "@flow/features/Canvas";
import type { YWorkflow } from "@flow/lib/yjs/types";

import { WorkflowTabs } from "../Editor/components";

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
    handleWorkflowClose,
    handleCurrentWorkflowIdChange,
  } = useHooks({ yWorkflows });

  return (
    <div className="relative flex size-full flex-col">
      <Canvas
        readonly
        isSubworkflow={isSubworkflow}
        nodes={nodes}
        edges={edges}
      />
      <WorkflowTabs
        openWorkflows={openWorkflows}
        currentWorkflowId={currentWorkflowId}
        onWorkflowClose={handleWorkflowClose}
        onWorkflowChange={handleCurrentWorkflowIdChange}
      />
    </div>
  );
};

export default VersionCanvas;
