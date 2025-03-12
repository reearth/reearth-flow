import { createLazyFileRoute, useParams } from "@tanstack/react-router";
import { ReactFlowProvider, useReactFlow } from "@xyflow/react";
import { useEffect, useState } from "react";

import { LoadingSplashscreen } from "@flow/components";
import Editor from "@flow/features/Editor";
import {
  ProjectIdWrapper,
  WorkspaceIdWrapper,
} from "@flow/features/PageWrapper";
import { DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";
import { useFullscreen, useShortcuts } from "@flow/hooks";
import { useAuth } from "@flow/lib/auth";
import useYjsSetup from "@flow/lib/yjs/useYjsSetup";
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

  const { yWorkflows, isSynced, undoManager, undoTrackerActionWrapper } =
    useYjsSetup({
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
      undoTrackerActionWrapper={undoTrackerActionWrapper}
    />
  );
};
