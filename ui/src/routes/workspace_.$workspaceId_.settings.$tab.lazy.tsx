import { createLazyFileRoute } from "@tanstack/react-router";

import { WorkspaceIdWrapper } from "@flow/features/PageWrapper";
import { WorkspaceSettings } from "@flow/pages";

export const Route = createLazyFileRoute("/workspace/$workspaceId/settings/$tab")({
  component: () => (
    <WorkspaceIdWrapper>
      <WorkspaceSettings />,
    </WorkspaceIdWrapper>
  ),
});
