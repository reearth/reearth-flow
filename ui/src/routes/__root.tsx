import { createRootRoute } from "@tanstack/react-router";

import NotFoundPage from "@flow/features/NotFoundPage";
import { RootRoute } from "@flow/features/Root";

export const Route = createRootRoute({
  component: RootRoute,
  notFoundComponent: () => <NotFoundPage />,
});
