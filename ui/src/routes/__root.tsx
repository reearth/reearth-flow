import { createRootRoute, Outlet } from "@tanstack/react-router";
import { TanStackRouterDevtools } from "@tanstack/router-devtools";
import { ReactFlowProvider } from "reactflow";

import Dialog from "@flow/features/Dialog";
import { I18nProvider, TooltipProvider } from "@flow/providers";

export const Route = createRootRoute({
  component: () => (
    <>
      <I18nProvider>
        <TooltipProvider>
          <ReactFlowProvider>
            <Dialog />
            {/* {!isLoading && <Dialog />} */}
            <Outlet />
          </ReactFlowProvider>
        </TooltipProvider>
      </I18nProvider>
      <TanStackRouterDevtools position="bottom-right" />
    </>
  ),
});
