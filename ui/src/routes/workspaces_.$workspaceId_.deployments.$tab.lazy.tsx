import { createLazyFileRoute } from "@tanstack/react-router";

import { Dashboard } from "@flow/features/Dashboard";
import { WorkspaceIdWrapper } from "@flow/features/PageWrapper";

export const Route = createLazyFileRoute(
  "/workspaces_/$workspaceId_/deployments/$tab",
)({
  component: () => (
    <WorkspaceIdWrapper>
      <Dashboard baseRoute="deployments" />
    </WorkspaceIdWrapper>
  ),
});
