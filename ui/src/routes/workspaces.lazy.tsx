import { createLazyFileRoute, Outlet } from "@tanstack/react-router";

import { WorkspaceIdWrapper } from "@flow/features/PageWrapper";
import LeftPanel from "@flow/features/WorkspaceLeftPanel";
import { TopNavigation } from "@flow/features/WorkspaceTopNavigation";

export const Route = createLazyFileRoute("/workspaces")({
  component: () => (
    <WorkspaceIdWrapper>
      <div className="flex h-screen flex-col">
        <TopNavigation />
        <div className="flex h-[calc(100vh-57px)] flex-1">
          <LeftPanel />
          <Outlet />
        </div>
      </div>
    </WorkspaceIdWrapper>
  ),
});
