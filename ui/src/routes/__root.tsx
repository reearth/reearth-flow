import { createRootRoute, Outlet } from "@tanstack/react-router";
import { lazy } from "react";

import { TooltipProvider } from "@flow/components";
import { config } from "@flow/config";
import AuthenticationWrapper from "@flow/features/AuthenticationWrapper";
import Dialog from "@flow/features/Dialog";
import NotFoundPage from "@flow/features/NotFoundPage";
import { AuthProvider } from "@flow/lib/auth";
import { GraphQLProvider } from "@flow/lib/gql";
import { I18nProvider } from "@flow/lib/i18n";

export const Route = createRootRoute({
  component: RootRoute,
  notFoundComponent: () => <NotFoundPage />,
});

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

function RootRoute() {
  const { devMode } = config();

  return (
    <AuthProvider>
      <GraphQLProvider>
        <I18nProvider>
          <TooltipProvider>
            <AuthenticationWrapper>
              <Dialog />
              <Outlet />
              {devMode && (
                <>
                  <TanStackQueryDevtools initialIsOpen={false} />
                  {/* <TanStackRouterDevtools /> */}
                </>
              )}
            </AuthenticationWrapper>
          </TooltipProvider>
        </I18nProvider>
      </GraphQLProvider>
    </AuthProvider>
  );
}
