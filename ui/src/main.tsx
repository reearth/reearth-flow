import { createRouter } from "@tanstack/react-router";
import { createRoot } from "react-dom/client";

import { App } from "@flow/App";
import loadConfig, { config } from "@flow/config";
import { AuthProvider } from "@flow/lib/auth";
import { enableMocking } from "@flow/mocks";
import { routeTree } from "@flow/routeTree.gen.ts";

import "@flow/index.css";
import NotFound from "./features/NotFound";
import { openDatabase } from "./stores";

const router = createRouter({
  routeTree,
  notFoundMode: "root",
  defaultNotFoundComponent: () => <NotFound />,
});

loadConfig().finally(async () => {
  // Enable mock server if configured
  const flowConfig = config();
  const enableMock = flowConfig.mockEnabled;

  if (enableMock) {
    console.log("🚀 Starting Mock Server for Re:Earth Flow");
    try {
      await enableMocking({ disabled: false });
    } catch (err) {
      // Starting the mock server is a dev convenience; it must not prevent the app
      // from mounting. This await used to throw straight out of the callback, so
      // React never rendered and the only symptom was a blank page with an
      // "Uncaught (in promise)" that is easy to miss. Service Worker registration
      // fails in some browsers and embedded webviews, which is exactly when a
      // developer most needs the page to load and say why.
      console.error(
        "Mock server failed to start; continuing without it. GraphQL requests will hit the real API.",
        err,
      );
    }
  }

  const element = document.getElementById("root");
  if (!element) throw new Error("root element is not found");

  // setup indexedDB with default state
  await openDatabase();

  const root = createRoot(element);
  root.render(
    <AuthProvider>
      <App router={router} />
    </AuthProvider>,
  );
});
