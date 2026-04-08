import { useParams, createFileRoute } from "@tanstack/react-router";
import { ReactFlowProvider, useReactFlow } from "@xyflow/react";
import { useEffect, useState } from "react";
import { useHotkeys } from "react-hotkeys-hook";

import { LoadingSplashscreen } from "@flow/components";
import ErrorPage from "@flow/components/errors/ErrorPage";
import Editor from "@flow/features/Editor";
import { WorkspaceIdWrapper } from "@flow/features/PageWrapper";
import {
  DEFAULT_ENTRY_GRAPH_ID,
  GLOBAL_HOT_KEYS,
} from "@flow/global-constants";
import { useFullscreen, useJobSubscriptionsSetup } from "@flow/hooks";
import { useAuth } from "@flow/lib/auth";
import useYjsSetup from "@flow/lib/yjs/useYjsSetup";

export const Route = createFileRoute(
  "/workspaces/$workspaceId_/projects_/$projectId_/debug_/$debugId",
)({
  component: () => (
    <WorkspaceIdWrapper>
      <ReactFlowProvider>
        <EditorComponent />
      </ReactFlowProvider>
    </WorkspaceIdWrapper>
  ),
  errorComponent: () => <ErrorPage />,
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

  const { projectId, debugId }: { projectId: string; debugId: string } =
    useParams({
      strict: false,
    });

  console.log("DEBUG ID", debugId);

  useJobSubscriptionsSetup(accessToken, debugId);

  const {
    yWorkflows,
    isSynced,
    undoTrackerActionWrapper,
    yDocState,
    yAwareness,
  } = useYjsSetup({
    isProtected: true,
    projectId,
    workflowId: DEFAULT_ENTRY_GRAPH_ID,
  });

  return !yWorkflows ||
    !isSynced ||
    !undoTrackerActionWrapper ||
    !yAwareness ? (
    <LoadingSplashscreen />
  ) : (
    <Editor
      yWorkflows={yWorkflows}
      undoManager={null}
      yDoc={yDocState}
      yAwareness={yAwareness}
      undoTrackerActionWrapper={undoTrackerActionWrapper}
    />
  );
};
