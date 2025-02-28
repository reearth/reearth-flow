import { createFileRoute } from "@tanstack/react-router";

import { DeploymentManager } from "@flow/features/WorkspaceDeployments";

export const Route = createFileRoute(
  "/workspaces/$workspaceId/deployments/$tab",
)({
  component: () => <DeploymentManager />,
});
