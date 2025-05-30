import { ArrowSquareIn, Export } from "@phosphor-icons/react";
import { useMemo } from "react";
import { Doc, Map as YMap } from "yjs";

import {
  IconButton,
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@flow/components";
import Canvas from "@flow/features/Canvas";
import { useUser } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { YWorkflow } from "@flow/lib/yjs/types";
import { Project } from "@flow/types";

import { ParamsPanel, WorkflowTabs } from "../Editor/components";
import { EditorContextType, EditorProvider } from "../Editor/editorContext";

import ImportPopover from "./components/ImportPopover";
import useHooks from "./hooks";
import useSharedProjectImport from "./useSharedProjectImport";

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
    handleDialogClose,
    handleShowImportPopover,
    showDialog,
  } = useHooks({ yWorkflows, project, undoTrackerActionWrapper });

  const { selectedWorkspaceId, handleProjectImport, handleSelectWorkspace } =
    useSharedProjectImport({
      sharedYdoc: yDoc,
      sharedProject: project,
      token: accessToken,
    });

  const { useGetMeAndWorkspaces } = useUser();

  const { me, workspaces } = useGetMeAndWorkspaces();

  const editorContext = useMemo(
    (): EditorContextType => ({
      onNodeSettings: handleNodeSettings,
    }),
    [handleNodeSettings],
  );
  console.log("WORKSPACES", workspaces);
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
              <Popover
                open={showDialog === "import"}
                onOpenChange={(open) => {
                  if (!open) handleDialogClose();
                }}>
                <PopoverTrigger>
                  <IconButton
                    tooltipText={t("Import Project")}
                    tooltipOffset={6}
                    icon={<ArrowSquareIn weight="thin" size={18} />}
                    onClick={handleShowImportPopover}
                  />
                </PopoverTrigger>
                <PopoverContent>
                  {showDialog === "import" && workspaces && (
                    <ImportPopover
                      workspaces={workspaces}
                      selectedWorkspaceId={selectedWorkspaceId}
                      onSelectWorkspace={handleSelectWorkspace}
                      onImportProject={handleProjectImport}
                    />
                  )}
                </PopoverContent>
              </Popover>
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
