import { useMemo } from "react";
import { Doc, Map as YMap } from "yjs";

import Canvas from "@flow/features/Canvas";
import { useUser } from "@flow/lib/gql";
import type { YWorkflow } from "@flow/lib/yjs/types";
import type { Project } from "@flow/types";

import { ParamsDialog } from "../Editor/components";
import { EditorContextType, EditorProvider } from "../Editor/editorContext";

import { SharedCanvasTopBar } from "./components";
import useHooks from "./hooks";

type Props = {
  yWorkflows: YMap<YWorkflow>;
  project?: Project;
  yDoc: Doc | null;
  undoTrackerActionWrapper: (
    callback: () => void,
    originPrepend?: string,
  ) => void;
  accessToken?: string;
};

const SharedCanvas: React.FC<Props> = ({
  yWorkflows,
  yDoc,
  project,
  accessToken,
  undoTrackerActionWrapper,
}) => {
  const {
    currentWorkflowId,
    isSubworkflow,
    nodes,
    edges,
    openWorkflows,
    openNode,
    handleOpenNode,
    handleNodeSettings,
    handleWorkflowOpen,
    handleWorkflowClose,
    handleCurrentWorkflowIdChange,
  } = useHooks({ yWorkflows, undoTrackerActionWrapper });

  const { useGetMeAndWorkspaces } = useUser();

  const { me, workspaces } = useGetMeAndWorkspaces();

  const editorContext = useMemo(
    (): EditorContextType => ({
      onNodeSettings: handleNodeSettings,
    }),
    [handleNodeSettings],
  );
  return (
    <div className="flex h-screen flex-col">
      <EditorProvider value={editorContext}>
        <div className="flex h-[44px] w-[100vw] shrink-0 justify-between gap-2 bg-secondary">
          <SharedCanvasTopBar
            currentWorkflowId={currentWorkflowId}
            openWorkflows={openWorkflows}
            yDoc={yDoc}
            project={project}
            accessToken={accessToken}
            me={me}
            workspaces={workspaces}
            onWorkflowClose={handleWorkflowClose}
            onWorkflowChange={handleCurrentWorkflowIdChange}
          />
        </div>
        <div className="relative flex flex-1">
          <div className="relative flex flex-1 flex-col">
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

export default SharedCanvas;
