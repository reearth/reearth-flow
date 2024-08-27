import { createLazyFileRoute } from "@tanstack/react-router";
import { ReactFlowProvider, useReactFlow } from "@xyflow/react";

import Editor from "@flow/features/Editor";
import {
  ProjectIdWrapper,
  WorkspaceIdWrapper,
} from "@flow/features/PageWrapper";
import { useFullscreen, useShortcuts } from "@flow/hooks";
// import { useShortcut } from "@flow/hooks/useShortcut";

export const Route = createLazyFileRoute(
  "/workspace/$workspaceId/project/$projectId"
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

  return <Editor />;
};
