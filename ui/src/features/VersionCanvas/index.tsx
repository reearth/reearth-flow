import { useMemo } from "react";
import { Map as YMap } from "yjs";

import Canvas from "@flow/features/Canvas";
import type { YWorkflow } from "@flow/lib/yjs/types";

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
    openNode,
    handleWorkflowOpen,
    handleWorkflowClose,
    handleCurrentWorkflowIdChange,
    handleOpenNode,
    handleNodeSettings,
  } = useHooks({ yWorkflows });

  const editorContext = useMemo(
    (): EditorContextType => ({
      onNodeSettings: handleNodeSettings,
    }),
    [handleNodeSettings],
  );

  return (
    <div className="flex h-screen flex-col">
      <EditorProvider value={editorContext}>
        <div className="flex shrink-0 justify-between gap-2 bg-secondary h-[44px]  w-[100vw]">
          <WorkflowTabs
            openWorkflows={openWorkflows}
            currentWorkflowId={currentWorkflowId}
            onWorkflowClose={handleWorkflowClose}
            onWorkflowChange={handleCurrentWorkflowIdChange}
          />
        </div>
        <div className="relative flex flex-1">
          <div className="flex flex-1 flex-col relative">
            <Canvas
              readonly
              isSubworkflow={isSubworkflow}
              onWorkflowOpen={handleWorkflowOpen}
              nodes={nodes}
              edges={edges}
              onNodeSettings={handleNodeSettings}
            />
          </div>
        </div>
        <ParamsPanel readonly openNode={openNode} onOpenNode={handleOpenNode} />
      </EditorProvider>
    </div>
  );
};

export default VersionCanvas;
