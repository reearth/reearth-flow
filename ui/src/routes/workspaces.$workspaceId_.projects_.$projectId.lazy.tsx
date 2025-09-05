import { createLazyFileRoute, useParams } from "@tanstack/react-router";
import { ReactFlowProvider, useReactFlow } from "@xyflow/react";
import { useEffect, useMemo, useState } from "react";
import { useHotkeys } from "react-hotkeys-hook";

import { Button, FlowLogo, LoadingSplashscreen } from "@flow/components";
import BasicBoiler from "@flow/components/BasicBoiler";
import ErrorPage from "@flow/components/errors/ErrorPage";
import { ProjectCorruptionError } from "@flow/errors";
import Editor from "@flow/features/Editor";
import { VersionDialog } from "@flow/features/Editor/components/TopBar/components/ActionBar/components/Version/VersionDialog";
import {
  ProjectIdWrapper,
  WorkspaceIdWrapper,
} from "@flow/features/PageWrapper";
import {
  DEFAULT_ENTRY_GRAPH_ID,
  GLOBAL_HOT_KEYS,
} from "@flow/global-constants";
import { useFullscreen, useJobSubscriptionsSetup } from "@flow/hooks";
import { useAuth } from "@flow/lib/auth";
import { useT } from "@flow/lib/i18n";
import { useIndexedDB } from "@flow/lib/indexedDB";
import useYjsSetup from "@flow/lib/yjs/useYjsSetup";
import { useCurrentProject } from "@flow/stores";

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
  errorComponent: ({ error, reset }) => (
    <ErrorComponent error={error} onErrorReset={reset} />
  ),
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

  useHotkeys(
    GLOBAL_HOT_KEYS,
    (event, handler) => {
      const hasModifier = event.metaKey || event.ctrlKey;

      switch (handler.keys?.join("")) {
        case "equal":
          zoomIn();
          break;
        case "minus":
          zoomOut();
          break;
        case "0":
          fitView();
          break;
        case "f":
          if (hasModifier) handleFullscreenToggle();
          break;
      }
    },
    { preventDefault: true },
  );

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
    awareness,
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
      awareness={awareness}
      undoTrackerActionWrapper={undoTrackerActionWrapper}
    />
  );
};

const ErrorComponent = ({
  error,
  onErrorReset,
}: {
  error: Error;
  onErrorReset: () => void;
}) => {
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
      {error instanceof ProjectCorruptionError ? (
        <div className="flex h-screen w-full flex-col items-center justify-center">
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
          {openVersionDialog && (
            <VersionDialog
              yDoc={yDocState}
              project={currentProject}
              onDialogClose={handleCloseDialog}
              onErrorReset={onErrorReset}
            />
          )}
        </div>
      ) : (
        <ErrorPage />
      )}
    </>
  );
};
