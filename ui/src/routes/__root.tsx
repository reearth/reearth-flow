import { createRootRoute, Outlet } from "@tanstack/react-router";
// import { lazy } from "react";

import { TooltipProvider } from "@flow/components";
import { config } from "@flow/config";
import AuthenticationWrapper from "@flow/features/AuthenticationWrapper";
import NotFoundPage from "@flow/features/NotFound";
import { NotificationSystem } from "@flow/features/NotificationSystem";
import { AuthProvider } from "@flow/lib/auth";
import { GraphQLProvider } from "@flow/lib/gql";
import { I18nProvider } from "@flow/lib/i18n";
import { ThemeProvider } from "@flow/lib/theme";

export const Route = createRootRoute({
  component: RootRoute,
  notFoundComponent: () => <NotFoundPage />,
});

// const TanStackQueryDevtools = lazy(() =>
//   import("@tanstack/react-query-devtools/build/modern/production.js").then(
//     (d) => ({
//       default: d.ReactQueryDevtools,
//     }),
//   ),
// );

// const TanStackRouterDevtools = lazy(() =>
//   import("@tanstack/router-devtools").then((d) => ({
//     default: d.TanStackRouterDevtools,
//   })),
// );

function RootRoute() {
  const { devMode } = config();

  return (
    <AuthProvider>
      <AuthenticationWrapper>
        <ThemeProvider>
          <GraphQLProvider>
            <I18nProvider>
              <TooltipProvider>
                <NotificationSystem />
                <Outlet />
                {devMode && (
                  <>
                    {/* <TanStackQueryDevtools initialIsOpen={false} /> */}
                    {/* <TanStackRouterDevtools /> */}
                  </>
                )}
              </TooltipProvider>
            </I18nProvider>
          </GraphQLProvider>
        </ThemeProvider>
      </AuthenticationWrapper>
    </AuthProvider>
  );
}
