import { ArrowSquareIn, Export } from "@phosphor-icons/react";
import { useMemo } from "react";
import { Map as YMap } from "yjs";

import { IconButton } from "@flow/components";
import Canvas from "@flow/features/Canvas";
import { useUser } from "@flow/lib/gql";
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
    locallyLockedNode,
    // isMainWorkflow,
    // hoveredDetails,
    handleProjectExport,
    // handleNodeHover,
    // handleEdgeHover,
    handleNodeSettings,
    handleWorkflowOpen,
    handleWorkflowClose,
    handleCurrentWorkflowIdChange,
  } = useHooks({ yWorkflows, project, undoTrackerActionWrapper });

  const { useGetMeAndWorkspaces } = useUser();

  const { me } = useGetMeAndWorkspaces();
  console.log("USER INFO", me?.email);
  const editorContext = useMemo(
    (): EditorContextType => ({
      onNodeSettings: handleNodeSettings,
    }),
    [handleNodeSettings],
  );

  return (
    <div className="flex h-screen flex-col">
      <EditorProvider value={editorContext}>
        <div className="flex shrink-0 justify-between gap-2 bg-secondary h-[44px] w-[100vw]">
          <div className="flex flex-1 gap-2 h-full overflow-hidden">
            <WorkflowTabs
              currentWorkflowId={currentWorkflowId}
              openWorkflows={openWorkflows}
              onWorkflowClose={handleWorkflowClose}
              onWorkflowChange={handleCurrentWorkflowIdChange}
            />
          </div>
          <div className="flex items-center">
            <IconButton
              tooltipText={t("Export Project")}
              tooltipOffset={6}
              icon={<Export weight="thin" size={18} />}
              onClick={handleProjectExport}
            />
            {me && (
              <IconButton
                tooltipText={t("Import Project")}
                tooltipOffset={6}
                icon={<ArrowSquareIn weight="thin" size={18} />}
                onClick={handleProjectExport}
              />
            )}
          </div>
        </div>
        <div className="relative flex flex-1">
          <div className="flex flex-1 flex-col">
            <Canvas
              isSubworkflow={isSubworkflow}
              onWorkflowOpen={handleWorkflowOpen}
              nodes={nodes}
              edges={edges}
              canvasLock
              onNodeSettings={handleNodeSettings}
            />
          </div>
        </div>
        <ParamsPanel selected={locallyLockedNode} />
      </EditorProvider>
    </div>
  );
};

export default SharedCanvas;
