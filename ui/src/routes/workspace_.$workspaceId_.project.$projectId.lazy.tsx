import { createLazyFileRoute } from "@tanstack/react-router";

import { ProjectIdWrapper, WorkspaceIdWrapper } from "@flow/features/PageWrapper";
import { Editor } from "@flow/pages";

export const Route = createLazyFileRoute("/workspace/$workspaceId/project/$projectId")({
  component: () => (
    <WorkspaceIdWrapper>
      <ProjectIdWrapper>
        <Editor />
      </ProjectIdWrapper>
    </WorkspaceIdWrapper>
  ),
});
