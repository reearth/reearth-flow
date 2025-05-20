import { useMemo } from "react";
import { Map as YMap } from "yjs";

import Canvas from "@flow/features/Canvas";
import { YWorkflow } from "@flow/lib/yjs/types";

import { ParamsPanel, WorkflowTabs } from "../Editor/components";
import { EditorContextType, EditorProvider } from "../Editor/editorContext";

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
    // isMainWorkflow,
    // hoveredDetails,
    // handleNodeHover,
    // handleEdgeHover,
    handleNodeDoubleClick,
    // handleWorkflowOpen,
    handleWorkflowClose,
    handleCurrentWorkflowIdChange,
  } = useHooks({ yWorkflows });

  const editorContext = useMemo(
    (): EditorContextType => ({
      onSecondaryNodeAction: handleNodeDoubleClick,
    }),
    [handleNodeDoubleClick],
  );

  return (
    <div className="relative flex size-full flex-col">
      <EditorProvider value={editorContext}>
        <Canvas
          isSubworkflow={isSubworkflow}
          nodes={nodes}
          edges={edges}
          canvasLock
          // onNodeHover={handleNodeHover}
          onNodeDoubleClick={handleNodeDoubleClick}
          // onEdgeHover={handleEdgeHover}
        />
        <WorkflowTabs
          openWorkflows={openWorkflows}
          currentWorkflowId={currentWorkflowId}
          onWorkflowClose={handleWorkflowClose}
          onWorkflowChange={handleCurrentWorkflowIdChange}
        />
        <ParamsPanel selected={locallyLockedNode} />
      </EditorProvider>
    </div>
  );
};

export default VersionCanvas;
