import { createFileRoute } from "@tanstack/react-router";

import { TriggerManager } from "@flow/features/WorkspaceTriggers";

export const Route = createFileRoute("/workspaces/$workspaceId/triggers")({
  component: () => <TriggerManager />,
});
