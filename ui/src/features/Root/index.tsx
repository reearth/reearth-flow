import { Outlet, useParams } from "@tanstack/react-router";
import { lazy } from "react";
import { ReactFlowProvider } from "reactflow";

import { config } from "@flow/config";
import AuthenticatedPage from "@flow/features/AuthenticatedPage";
import Dialog from "@flow/features/Dialog";
import { AuthProvider } from "@flow/lib/auth";
import { GraphQLProvider } from "@flow/lib/gql";
import { workspaces } from "@flow/mock_data/workspaceData";
import { I18nProvider, TooltipProvider } from "@flow/providers";
import { useCurrentProject } from "@flow/stores";

const TanStackQueryDevtools = lazy(() =>
  import("@tanstack/react-query-devtools/build/modern/production.js").then(d => ({
    default: d.ReactQueryDevtools,
  })),
);

// const TanStackRouterDevtools = lazy(() =>
//   import("@tanstack/router-devtools").then(d => ({
//     default: d.TanStackRouterDevtools,
//   })),
// );

const RootRoute: React.FC = () => {
  const [currentProject, setCurrentProject] = useCurrentProject();

  const { devMode } = config();

  const { projectId } = useParams({ strict: false });

  if ((projectId && !currentProject) || projectId !== currentProject?.id) {
    setCurrentProject(workspaces[0].projects?.find(p => p.id === projectId));
  }

  return (
    <AuthProvider>
      <GraphQLProvider>
        <I18nProvider>
          <TooltipProvider>
            <ReactFlowProvider>
              <AuthenticatedPage>
                <Dialog />
                <Outlet />
                {devMode && (
                  <>
                    <TanStackQueryDevtools initialIsOpen={false} />
                    {/* <TanStackRouterDevtools /> */}
                  </>
                )}
              </AuthenticatedPage>
            </ReactFlowProvider>
          </TooltipProvider>
        </I18nProvider>
      </GraphQLProvider>
    </AuthProvider>
  );
};

export { RootRoute };
