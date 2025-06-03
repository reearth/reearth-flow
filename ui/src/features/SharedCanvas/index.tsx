import { useMemo } from "react";
import { Doc, Map as YMap } from "yjs";

import { FlowLogo } from "@flow/components";
import Canvas from "@flow/features/Canvas";
import { useUser } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import type { YWorkflow } from "@flow/lib/yjs/types";
import type { Project } from "@flow/types";

import { ParamsPanel } from "../Editor/components";
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
  const t = useT();
  return (
    <div className="flex h-screen flex-col">
      <EditorProvider value={editorContext}>
        <div className="flex shrink-0 justify-between gap-2 bg-secondary h-[44px] w-[100vw]">
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
          <div className="flex flex-1 flex-col relative">
            <Canvas
              readonly
              isSubworkflow={isSubworkflow}
              onWorkflowOpen={handleWorkflowOpen}
              nodes={nodes}
              edges={edges}
              onNodeSettings={handleNodeSettings}
            />
            <div className="absolute bottom-4 right-4 flex items-center justify-end gap-4 cursor-default select-none">
              <FlowLogo className="text-[#00A34188] size-10" />
              <p className="font-extralight text-gray-400 ">{t("Shared")}</p>
            </div>
          </div>
        </div>
        <ParamsPanel readonly openNode={openNode} onOpenNode={handleOpenNode} />
      </EditorProvider>
    </div>
  );
};

export default SharedCanvas;
