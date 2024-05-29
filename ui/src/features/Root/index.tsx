import { Outlet, useParams } from "@tanstack/react-router";
import { ReactFlowProvider } from "reactflow";

import AuthenticatedPage from "@flow/features/AuthenticatedPage";
import Dialog from "@flow/features/Dialog";
import { AuthProvider } from "@flow/lib/auth";
import { workspaces } from "@flow/mock_data/workspaceData";
import {
  GraphQlSdkProvider,
  I18nProvider,
  QueryClientProvider,
  TooltipProvider,
} from "@flow/providers";
import { useCurrentProject, useCurrentWorkspace } from "@flow/stores";
// import { lazy } from "react";

// import { config } from "@flow/config";

// const TanStackQueryDevtools = lazy(() =>
//   import("@tanstack/react-query-devtools/build/modern/production.js").then(d => ({
//     default: d.ReactQueryDevtools,
//   })),
// );

// const TanStackRouterDevtools = lazy(() =>
//   import("@tanstack/router-devtools").then(d => ({
//     default: d.TanStackRouterDevtools,
//   })),
// );

const RootRoute: React.FC = () => {
  const [currentProject, setCurrentProject] = useCurrentProject();
  const [currentWorkspace, setCurrentWorkspace] = useCurrentWorkspace();

  // const { devMode } = config();

  const { projectId, workspaceId } = useParams({ strict: false });

  if ((workspaceId && !currentWorkspace) || workspaceId !== currentWorkspace?.id) {
    setCurrentWorkspace(workspaces.find(w => w.id === workspaceId));
  }

  if (currentWorkspace && ((projectId && !currentProject) || projectId !== currentProject?.id)) {
    setCurrentProject(currentWorkspace.projects?.find(p => p.id === projectId));
  }

  return (
    <AuthProvider>
      <QueryClientProvider>
        <I18nProvider>
          <TooltipProvider>
            <ReactFlowProvider>
              <AuthenticatedPage>
                <GraphQlSdkProvider>
                  <Dialog />
                  <Outlet />
                </GraphQlSdkProvider>
              </AuthenticatedPage>
            </ReactFlowProvider>
          </TooltipProvider>
        </I18nProvider>
        {/* {devMode && (
          <>
            <TanStackQueryDevtools initialIsOpen={false} />
            <TanStackRouterDevtools />
          </>
        )} */}
      </QueryClientProvider>
    </AuthProvider>
  );
};

export { RootRoute };
