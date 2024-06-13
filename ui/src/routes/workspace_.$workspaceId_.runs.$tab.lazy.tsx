import { createLazyFileRoute } from "@tanstack/react-router";

import { WorkspaceRuns } from "@flow/pages";

export const Route = createLazyFileRoute("/workspace/$workspaceId/runs/$tab")({
  component: () => <WorkspaceRuns />,
});
