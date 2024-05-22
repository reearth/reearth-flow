import { RouterProvider, createRouter } from "@tanstack/react-router";
import { createRoot } from "react-dom/client";

import { AuthProvider, AuthenticatedPage } from "./auth/index.tsx";
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
      {/* TODO: Not the correct way but works. Need to refactor this. */}
      <AuthenticatedPage>
        <RouterProvider router={router} />
      </AuthenticatedPage>
    </AuthProvider>,
  );
});
