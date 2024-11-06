import { createFileRoute } from "@tanstack/react-router";

import WorkspaceSettings from "@flow/features/WorkspaceSettings";

export const Route = createFileRoute("/workspaces/$workspaceId_/settings/$tab")(
  {
    component: () => <WorkspaceSettings />,
  },
);
