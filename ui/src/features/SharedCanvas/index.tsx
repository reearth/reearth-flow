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
  undoTrackerActionWrapper: (
    callback: () => void,
    originPrepend?: string,
  ) => void;
};

const SharedCanvas: React.FC<Props> = ({
  yWorkflows,
  project,
  undoTrackerActionWrapper,
}) => {
  const t = useT();
  const {
    currentWorkflowId,
    isSubworkflow,
    nodes,
    edges,
    openWorkflows,
    openNode,
    // isMainWorkflow,
    // hoveredDetails,
    handleProjectExport,
    // handleNodeHover,
    // handleEdgeHover,
    handleOpenNode,
    handleNodeSettings,
    // handleWorkflowOpen,
    handleWorkflowClose,
    handleCurrentWorkflowIdChange,
  } = useHooks({ yWorkflows, project, undoTrackerActionWrapper });

  const editorContext = useMemo(
    (): EditorContextType => ({
      onNodeSettings: handleNodeSettings,
    }),
    [handleNodeSettings],
  );

  return (
    <div className="relative flex size-full flex-col">
      <EditorProvider value={editorContext}>
        <Canvas
          isSubworkflow={isSubworkflow}
          nodes={nodes}
          edges={edges}
          onNodeSettings={handleNodeSettings}
        />
        <WorkflowTabs
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
        {openNode && (
          <ParamsPanel openNode={openNode} onOpenNode={handleOpenNode} />
        )}
      </EditorProvider>
    </div>
  );
};

export default SharedCanvas;
