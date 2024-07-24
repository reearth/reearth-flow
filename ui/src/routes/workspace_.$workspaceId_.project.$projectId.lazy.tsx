import { createLazyFileRoute } from "@tanstack/react-router";
import { ReactFlowProvider } from "@xyflow/react";

import Canvas from "@flow/features/Editor";
import { ProjectIdWrapper, WorkspaceIdWrapper } from "@flow/features/PageWrapper";
import { useCurrentProject } from "@flow/stores";

export const Route = createLazyFileRoute("/workspace/$workspaceId/project/$projectId")({
  component: () => (
    <WorkspaceIdWrapper>
      <ProjectIdWrapper>
        <Editor />
      </ProjectIdWrapper>
    </WorkspaceIdWrapper>
  ),
});

function Editor() {
  const [currentProject] = useCurrentProject();

  return (
    <div className="flex h-screen flex-col">
      <ReactFlowProvider>
        <Canvas workflows={currentProject?.workflows} />
      </ReactFlowProvider>
    </div>
  );
}
