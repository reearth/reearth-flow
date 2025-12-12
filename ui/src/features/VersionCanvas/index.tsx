import { useMemo } from "react";
import { Map as YMap } from "yjs";

import Canvas from "@flow/features/Canvas";
import type { YWorkflow } from "@flow/lib/yjs/types";

import { ParamsDialog } from "../Editor/components";
import { EditorContextType, EditorProvider } from "../Editor/editorContext";

import VersionCanvasHomeMenu from "./components/VersionCanvasHomeMenu";
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
    isMainWorkflow,
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
        <div className="flex flex-1 flex-col">
          <div
            className={`relative flex flex-1 flex-col border ${isMainWorkflow ? "border-transparent" : "border-node-subworkflow"}`}>
            <div
              id="left-top"
              className="pointer-events-none absolute top-2 left-2 z-50 *:pointer-events-auto">
              <VersionCanvasHomeMenu
                currentWorkflowId={currentWorkflowId}
                openWorkflows={openWorkflows}
                onWorkflowClose={handleWorkflowClose}
                onWorkflowChange={handleCurrentWorkflowIdChange}
              />
            </div>
            <div className="flex flex-1">
              <div className="relative flex flex-1 flex-col">
                <Canvas
                  isMainWorkflow={isMainWorkflow}
                  readonly
                  onWorkflowOpen={handleWorkflowOpen}
                  nodes={nodes}
                  edges={edges}
                  onNodeSettings={handleNodeSettings}
                />
              </div>
            </div>
            {openNode && (
              <ParamsDialog
                readonly
                openNode={openNode}
                onOpenNode={handleOpenNode}
              />
            )}
          </div>
        </div>
      </EditorProvider>
    </div>
  );
};

export default VersionCanvas;
