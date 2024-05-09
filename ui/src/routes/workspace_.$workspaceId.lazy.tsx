import { createLazyFileRoute } from "@tanstack/react-router";

import { Dashboard } from "@flow/features/Dashboard";

export const Route = createLazyFileRoute("/workspace/$workspaceId")({
  component: () => <Dashboard />,
});
