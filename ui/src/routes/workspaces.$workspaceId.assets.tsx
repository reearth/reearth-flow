import { createFileRoute } from "@tanstack/react-router";

import { AssetsManager } from "@flow/features/WorkspaceAssets";

export const Route = createFileRoute("/workspaces/$workspaceId/assets")({
  component: () => <AssetsManager />,
});
