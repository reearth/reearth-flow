import { createLazyFileRoute } from "@tanstack/react-router";

import { WorkspaceIdWrapper } from "@flow/features/PageWrapper";
import { Runs } from "@flow/features/Runs";

export const Route = createLazyFileRoute("/workspace/$workspaceId/runs/$tab")({
  component: () => (
    <WorkspaceIdWrapper>
      <Runs />
    </WorkspaceIdWrapper>
  ),
});
