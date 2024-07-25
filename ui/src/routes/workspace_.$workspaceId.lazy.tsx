import { createLazyFileRoute } from "@tanstack/react-router";

import { Dashboard } from "@flow/features/Dashboard";
import { WorkspaceIdWrapper } from "@flow/features/PageWrapper";

export const Route = createLazyFileRoute("/workspace/$workspaceId")({
  component: () => (
    <WorkspaceIdWrapper>
      <Dashboard />
    </WorkspaceIdWrapper>
  ),
});
