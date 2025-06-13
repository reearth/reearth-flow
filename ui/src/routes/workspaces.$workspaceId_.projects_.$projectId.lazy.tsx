import { createLazyFileRoute, useParams } from "@tanstack/react-router";
import { ReactFlowProvider, useReactFlow } from "@xyflow/react";
import { useEffect, useMemo, useState } from "react";

import { Button, FlowLogo, LoadingSplashscreen } from "@flow/components";
import BasicBoiler from "@flow/components/BasicBoiler";
import Editor from "@flow/features/Editor";
import { VersionDialog } from "@flow/features/Editor/components/TopBar/components/ActionBar/components/Version/VersionDialog";
import {
  ProjectIdWrapper,
  WorkspaceIdWrapper,
} from "@flow/features/PageWrapper";
import { DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";
import {
  useJobSubscriptionsSetup,
  useFullscreen,
  useShortcuts,
} from "@flow/hooks";
import { useAuth } from "@flow/lib/auth";
import { useT } from "@flow/lib/i18n";
import { useIndexedDB } from "@flow/lib/indexedDB";
import useYjsSetup from "@flow/lib/yjs/useYjsSetup";
import { useCurrentProject } from "@flow/stores";
// import { useShortcut } from "@flow/hooks/useShortcut";

export const Route = createLazyFileRoute(
  "/workspaces/$workspaceId_/projects_/$projectId",
)({
  component: () => (
    <WorkspaceIdWrapper>
      <ProjectIdWrapper>
        <ReactFlowProvider>
          <EditorComponent />
        </ReactFlowProvider>
      </ProjectIdWrapper>
    </WorkspaceIdWrapper>
  ),
  errorComponent: ({ reset }) => <ErrorComponent onErrorReset={reset} />,
});

const EditorComponent = () => {
  const { zoomIn, zoomOut, fitView } = useReactFlow();
  const { handleFullscreenToggle } = useFullscreen();

  const [accessToken, setAccessToken] = useState<string | undefined>(undefined);

  const { getAccessToken } = useAuth();

  useEffect(() => {
    if (!accessToken) {
      (async () => {
        const token = await getAccessToken();
        setAccessToken(token);
      })();
    }
  }, [accessToken, getAccessToken]);

  useShortcuts([
    {
      keyBinding: { key: "+", commandKey: false },
      callback: zoomIn,
    },
    {
      keyBinding: { key: "-", commandKey: false },
      callback: zoomOut,
    },
    {
      keyBinding: { key: "0", commandKey: true },
      callback: fitView,
    },
    {
      keyBinding: { key: "f", commandKey: true },
      callback: handleFullscreenToggle,
    },
  ]);

  const { projectId }: { projectId: string } = useParams({
    strict: false,
  });

  const [currentProject] = useCurrentProject();
  const { value: debugRunState } = useIndexedDB("debugRun");

  const currentDebugJobId = useMemo(
    () =>
      debugRunState?.jobs?.find((job) => job.projectId === currentProject?.id)
        ?.jobId,
    [debugRunState, currentProject],
  );

  useJobSubscriptionsSetup(accessToken, currentDebugJobId, currentProject?.id);

  const {
    yWorkflows,
    isSynced,
    undoManager,
    undoTrackerActionWrapper,
    yDocState,
  } = useYjsSetup({
    isProtected: true,
    projectId,
    workflowId: DEFAULT_ENTRY_GRAPH_ID,
  });

  return !yWorkflows || !isSynced || !undoTrackerActionWrapper ? (
    <LoadingSplashscreen />
  ) : (
    <Editor
      yWorkflows={yWorkflows}
      undoManager={undoManager}
      yDoc={yDocState}
      undoTrackerActionWrapper={undoTrackerActionWrapper}
    />
  );
};

const ErrorComponent = ({ onErrorReset }: { onErrorReset: () => void }) => {
  const [openVersionDialog, setOpenVersionDialog] = useState(false);
  const t = useT();

  const [currentProject] = useCurrentProject();

  const { yDocState } = useYjsSetup({
    isProtected: true,
    projectId: currentProject?.id,
    workflowId: DEFAULT_ENTRY_GRAPH_ID,
  });

  const handleCloseDialog = () => {
    setOpenVersionDialog(false);
  };
  return (
    <>
      <div className="flex flex-col h-screen w-full items-center justify-center">
        <div className="flex flex-col items-center justify-center gap-8">
          <BasicBoiler
            text={t("Project or version is corrupted.")}
            icon={<FlowLogo className="size-16 text-accent" />}
          />
          <Button
            onClick={() => setOpenVersionDialog(true)}
            variant="default"
            className="ml-4">
            {t("Revert to a previous version")}
          </Button>
        </div>
      </div>
      {openVersionDialog && (
        <VersionDialog
          yDoc={yDocState}
          project={currentProject}
          onDialogClose={handleCloseDialog}
          onErrorReset={onErrorReset}
        />
      )}
    </>
  );
};
