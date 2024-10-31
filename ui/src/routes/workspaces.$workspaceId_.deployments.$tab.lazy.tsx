import { createLazyFileRoute } from "@tanstack/react-router";

import { DeploymentManager } from "@flow/features/WorkspaceDeployments";

export const Route = createLazyFileRoute(
  "/workspaces/$workspaceId_/deployments/$tab",
)({
  component: () => <DeploymentManager />,
});
