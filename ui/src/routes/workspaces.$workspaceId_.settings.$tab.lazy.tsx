import { createLazyFileRoute } from "@tanstack/react-router";

import WorkspaceSettings from "@flow/features/WorkspaceSettings";

export const Route = createLazyFileRoute(
  "/workspaces/$workspaceId_/settings/$tab",
)({
  component: () => <WorkspaceSettings />,
});
