import { createLazyFileRoute } from "@tanstack/react-router";

import { WorkspaceSettings } from "@flow/pages";

export const Route = createLazyFileRoute("/workspace/$workspaceId/settings/$tab")({
  component: () => <WorkspaceSettings />,
});
