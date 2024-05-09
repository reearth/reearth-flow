import { createRootRoute, Outlet, useParams } from "@tanstack/react-router";
import { lazy } from "react";
import { ReactFlowProvider } from "reactflow";

import { config } from "@flow/config";
import Dialog from "@flow/features/Dialog";
import { workspaces } from "@flow/mock_data/workspaceData";
import { I18nProvider, QueryClientProvider, TooltipProvider } from "@flow/providers";
import { useCurrentProject, useCurrentWorkspace } from "@flow/stores";
import NotFoundPage from "@flow/features/NotFoundPage";

const TanStackQueryDevtools = lazy(() =>
  import("@tanstack/react-query-devtools/build/modern/production.js").then(d => ({
    default: d.ReactQueryDevtools,
  })),
);

const TanStackRouterDevtools = lazy(() =>
  import("@tanstack/router-devtools").then(d => ({
    default: d.TanStackRouterDevtools,
  })),
);

const RootRoute: React.FC = () => {
  const [currentProject, setCurrentProject] = useCurrentProject();
  const [currentWorkspace, setCurrentWorkspace] = useCurrentWorkspace();

  const { devMode } = config();

  const { projectId, workspaceId } = useParams({ strict: false });

  if ((workspaceId && !currentWorkspace) || workspaceId !== currentWorkspace?.id) {
    setCurrentWorkspace(workspaces.find(w => w.id === workspaceId));
  }

  if (currentWorkspace && ((projectId && !currentProject) || projectId !== currentProject?.id)) {
    setCurrentProject(currentWorkspace.projects?.find(p => p.id === projectId));
  }

  return (
    <QueryClientProvider>
      <I18nProvider>
        <TooltipProvider>
          <ReactFlowProvider>
            <Dialog />
            <Outlet />
          </ReactFlowProvider>
        </TooltipProvider>
      </I18nProvider>
      {devMode && (
        <>
          <TanStackQueryDevtools initialIsOpen={false} />
          <TanStackRouterDevtools />
        </>
      )}
    </QueryClientProvider>
  );
};

export const Route = createRootRoute({
  component: RootRoute,
  notFoundComponent: () => <NotFoundPage />,
});
