import { useMemo } from "react";

import Canvas from "@flow/features/Canvas";

import { ParamsDialog } from "../Editor/components";
import type { EditorContextType } from "../Editor/editorContext";
import { EditorProvider } from "../Editor/editorContext";

import EmbedStatus from "./components/EmbedStatus";
import EmbedZoomControls from "./components/EmbedZoomControls";
import useHooks from "./hooks";

type Props = {
  projectId: string;
};

const EmbedCanvas: React.FC<Props> = ({ projectId }) => {
  const {
    nodes,
    edges,
    error,
    isLoading,
    isEmpty,
    isMainWorkflow,
    openNode,
    handleWorkflowOpen,
    handleOpenNode,
    handleNodeSettings,
  } = useHooks({ projectId });

  const editorContext = useMemo(
    (): EditorContextType => ({
      isLocked: true,
      isReaderRestricted: false,
      canViewIntermediateData: false,
      onNodeSettings: handleNodeSettings,
    }),
    [handleNodeSettings],
  );

  const status = error
    ? ({ status: "error", reason: error } as const)
    : isLoading
      ? ({ status: "loading" } as const)
      : isEmpty
        ? ({ status: "empty" } as const)
        : undefined;

  return (
    <EditorProvider value={editorContext}>
      <div
        className={`relative flex h-screen w-full flex-col border ${isMainWorkflow ? "border-transparent" : "border-node-subworkflow"}`}>
        {status ? (
          <div className="flex flex-1 items-center justify-center bg-background">
            <EmbedStatus {...status} />
          </div>
        ) : (
          <>
            <Canvas
              isMainWorkflow={isMainWorkflow}
              simpleEdges
              nodes={nodes}
              edges={edges}
              onWorkflowOpen={handleWorkflowOpen}
              onNodeSettings={handleNodeSettings}
            />
            <div className="absolute right-4 bottom-4 z-10">
              <EmbedZoomControls />
            </div>
          </>
        )}
        {openNode && (
          <ParamsDialog openNode={openNode} onOpenNode={handleOpenNode} />
        )}
      </div>
    </EditorProvider>
  );
};

export default EmbedCanvas;
