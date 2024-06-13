import { createLazyFileRoute } from "@tanstack/react-router";

import { WorkspaceIdWrapper } from "@flow/features/PageWrapper";
import { WorkspaceRuns } from "@flow/pages";

export const Route = createLazyFileRoute("/workspace/$workspaceId/runs/$tab")({
  component: () => (
    <WorkspaceIdWrapper>
      <WorkspaceRuns />
    </WorkspaceIdWrapper>
  ),
});
