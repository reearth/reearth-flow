import { useMemo } from "react";
import { Map as YMap } from "yjs";

import Canvas from "@flow/features/Canvas";
import type { YWorkflow } from "@flow/lib/yjs/types";

import { ParamsDialog, WorkflowTabs } from "../Editor/components";
import { EditorContextType, EditorProvider } from "../Editor/editorContext";

import useHooks from "./hooks";

type Props = {
  yWorkflows: YMap<YWorkflow>;
};

const VersionCanvas: React.FC<Props> = ({ yWorkflows }) => {
  const {
    currentWorkflowId,
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
      onNodeSettings:
        handleNodeSettings as unknown as EditorContextType["onNodeSettings"],
    }),
    [handleNodeSettings],
  );

  return (
    <div className="flex h-full flex-col">
      <EditorProvider value={editorContext}>
        <div className="h-[42px] w-full bg-secondary">
          <WorkflowTabs
            openWorkflows={openWorkflows}
            currentWorkflowId={currentWorkflowId}
            onWorkflowClose={handleWorkflowClose}
            onWorkflowChange={handleCurrentWorkflowIdChange}
          />
        </div>
        <div className="relative flex flex-1">
          <Canvas
            readonly
            onWorkflowOpen={handleWorkflowOpen}
            nodes={nodes}
            edges={edges}
            onNodeSettings={handleNodeSettings}
          />
        </div>
        {openNode && (
          <ParamsDialog
            readonly
            openNode={openNode}
            onOpenNode={handleOpenNode}
          />
        )}
      </EditorProvider>
    </div>
  );
};

export default VersionCanvas;
