import { RouterProvider as TanStackRouterProvider, createRouter } from "@tanstack/react-router";

import { routeTree } from "@flow/routeTree.gen";

const router = createRouter({ routeTree });

const RouterProvider = () => {
  return <TanStackRouterProvider router={router} />;
};

export { RouterProvider };
