import { Outlet } from "@tanstack/react-router";
import { lazy } from "react";
import { ReactFlowProvider } from "reactflow";

import { TooltipProvider } from "@flow/components";
import { config } from "@flow/config";
import AuthenticatedPage from "@flow/features/AuthenticatedPage";
import Dialog from "@flow/features/Dialog";
import { AuthProvider } from "@flow/lib/auth";
import { GraphQLProvider } from "@flow/lib/gql";
import { I18nProvider } from "@flow/lib/i18n";

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
  const { devMode } = config();

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
