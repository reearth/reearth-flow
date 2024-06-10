import { createLazyFileRoute } from "@tanstack/react-router";

import { Runs } from "@flow/features/Runs/";

export const Route = createLazyFileRoute("/workspace/$workspaceId/runs/$tab")({
  component: () => <Runs />,
});
