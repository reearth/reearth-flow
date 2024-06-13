import { createLazyFileRoute } from "@tanstack/react-router";

import { WorkspaceIdWrapper } from "@flow/features/PageWrapper";
import { Dashboard } from "@flow/pages";

export const Route = createLazyFileRoute("/workspace/$workspaceId")({
  component: () => (
    <WorkspaceIdWrapper>
      <Dashboard />
    </WorkspaceIdWrapper>
  ),
});
