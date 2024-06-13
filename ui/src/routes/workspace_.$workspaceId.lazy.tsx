import { createLazyFileRoute } from "@tanstack/react-router";

import { Dashboard } from "@flow/pages";

export const Route = createLazyFileRoute("/workspace/$workspaceId")({
  component: () => <Dashboard />,
});
