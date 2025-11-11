import { PaperPlaneTiltIcon, QuestionIcon } from "@phosphor-icons/react";
import { useMemo } from "react";
import { Doc, Map as YMap } from "yjs";

import { Tooltip, TooltipContent, TooltipTrigger } from "@flow/components";
import Canvas from "@flow/features/Canvas";
import { useUser } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import type { YWorkflow } from "@flow/lib/yjs/types";
import type { Project } from "@flow/types";

import { ParamsDialog } from "../Editor/components";
import { EditorContextType, EditorProvider } from "../Editor/editorContext";

import { SharedCanvasActions, SharedCanvasHomeMenu } from "./components";
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
  const t = useT();

  const {
    currentWorkflowId,
    nodes,
    edges,
    openWorkflows,
    openNode,
    isMainWorkflow,
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
        <div className="flex flex-1 flex-col">
          <div
            className={`relative flex flex-1 flex-col border ${isMainWorkflow ? "border-transparent" : "border-node-subworkflow"}`}>
            <div
              id="left-top"
              className="pointer-events-none absolute top-2 left-2 z-50 *:pointer-events-auto">
              <SharedCanvasHomeMenu
                currentWorkflowId={currentWorkflowId}
                openWorkflows={openWorkflows}
                project={project}
                isMainWorkflow={isMainWorkflow}
                workspaces={workspaces}
                onWorkflowClose={handleWorkflowClose}
                onWorkflowChange={handleCurrentWorkflowIdChange}
              />
            </div>
            <div
              id="right-top"
              className="absolute top-2 right-2 z-50 h-[42px]">
              <div
                className={`flex h-full items-center justify-center gap-2 self-center rounded-xl border bg-secondary/70 p-1 shadow-md shadow-secondary backdrop-blur-xs select-none ${isMainWorkflow ? "border-primary" : "border-node-subworkflow"}`}>
                <SharedCanvasActions
                  yDoc={yDoc}
                  project={project}
                  accessToken={accessToken}
                  isMainWorkflow={isMainWorkflow}
                  me={me}
                  workspaces={workspaces}
                />
              </div>
            </div>
            <div className="flex flex-1">
              <div className="relative flex flex-1 flex-col">
                <div className="absolute bottom-4 left-4 z-10 flex shrink-0 justify-center">
                  <div className="flex items-center gap-2 rounded border border-logo/50 bg-logo/10 p-2">
                    <PaperPlaneTiltIcon weight="thin" size={18} />
                    <p className="font-light text-accent-foreground select-none">
                      {t("Shared Project")}
                    </p>
                    <Tooltip delayDuration={0}>
                      <TooltipTrigger>
                        <QuestionIcon weight="thin" size={14} />
                      </TooltipTrigger>
                      <TooltipContent className="max-w-[200px]" sideOffset={18}>
                        <div className="flex flex-col gap-2">
                          <div className="flex items-center gap-1">
                            <QuestionIcon size={12} />
                            <p>{t("Shared Project")}</p>
                          </div>
                          <p>
                            {t(
                              "A shared project is in a read only state. To start editing or to run this project, please import it into one of your workspaces.",
                            )}
                          </p>
                        </div>
                      </TooltipContent>
                    </Tooltip>
                  </div>
                </div>
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

export default SharedCanvas;
