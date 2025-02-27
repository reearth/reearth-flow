import { createRootRoute, Outlet } from "@tanstack/react-router";

import { config } from "@flow/config";
import NotFoundPage from "@flow/features/NotFound";
import { AuthProvider } from "@flow/lib/auth";

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
      <Outlet />
      {devMode && (
        <>
          {/* <TanStackQueryDevtools initialIsOpen={false} /> */}
          {/* <TanStackRouterDevtools /> */}
        </>
      )}
    </AuthProvider>
  );
}
