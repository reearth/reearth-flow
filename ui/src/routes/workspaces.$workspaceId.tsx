import { createFileRoute, Outlet } from "@tanstack/react-router";

import { WorkspaceIdWrapper } from "@flow/features/PageWrapper";
import LeftPanel from "@flow/features/WorkspaceLeftPanel";

export const Route = createFileRoute("/workspaces/$workspaceId")({
  component: () => (
    <WorkspaceIdWrapper>
      <div className="flex h-screen flex-col">
        <div className="flex h-[calc(100vh-57px)] flex-1">
          <LeftPanel />
          <Outlet />
        </div>
      </div>
    </WorkspaceIdWrapper>
  ),
});
