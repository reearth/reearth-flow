import { useMemo } from "react";
import { Map as YMap } from "yjs";

import { Button } from "@flow/components";
import Canvas from "@flow/features/Canvas";
import { useT } from "@flow/lib/i18n";
import { YWorkflow } from "@flow/lib/yjs/types";
import { Project } from "@flow/types";

import { ParamsPanel, WorkflowTabs } from "../Editor/components";
import { EditorContextType, EditorProvider } from "../Editor/editorContext";

import useHooks from "./hooks";

type Props = {
  yWorkflows: YMap<YWorkflow>;
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

  const editorContext = useMemo(
    (): EditorContextType => ({
      onNodesChange: handleNodesChange,
      onSecondaryNodeAction: handleNodeDoubleClick,
    }),
    [handleNodesChange, handleNodeDoubleClick],
  );

  return (
    <div className="relative flex size-full flex-col">
      <EditorProvider value={editorContext}>
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
      </EditorProvider>
    </div>
  );
};

export default SharedCanvas;
