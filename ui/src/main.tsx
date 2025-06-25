import { RouterProvider, createRouter } from "@tanstack/react-router";
import { createRoot } from "react-dom/client";

import loadConfig from "@flow/config";
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
  const element = document.getElementById("root");
  if (!element) throw new Error("root element is not found");

  // setup indexedDB with default state
  await openDatabase();

  const root = createRoot(element);
  root.render(<RouterProvider router={router} />);
});
