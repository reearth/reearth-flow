import { createFileRoute } from "@tanstack/react-router";

import { RunsManager } from "@flow/features/WorkspaceRuns";

export const Route = createFileRoute("/workspaces/$workspaceId_/runs/$tab")({
  component: () => <RunsManager />,
});
