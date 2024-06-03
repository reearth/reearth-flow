import { RouterProvider, createRouter } from "@tanstack/react-router";
import { createRoot } from "react-dom/client";
import { ReactFlowProvider } from "reactflow";

import AuthenticatedPage from "@flow/features/AuthenticatedPage";
import { AuthProvider } from "@flow/lib/auth";
import { GraphQLProvider } from "@flow/lib/gql";
import { I18nProvider, TooltipProvider } from "@flow/providers";

import loadConfig from "./config";
import { routeTree } from "./routeTree.gen.ts";
import "./index.css";

const router = createRouter({ routeTree });

loadConfig().finally(async () => {
  const element = document.getElementById("root");
  if (!element) throw new Error("root element is not found");

  const root = createRoot(element);
  root.render(
    <AuthProvider>
      <GraphQLProvider>
        <I18nProvider>
          <TooltipProvider>
            <ReactFlowProvider>
              <AuthenticatedPage>
                <RouterProvider router={router} />
              </AuthenticatedPage>
            </ReactFlowProvider>
          </TooltipProvider>
        </I18nProvider>
      </GraphQLProvider>
    </AuthProvider>,
  );
});
