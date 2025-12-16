import { createFileRoute, notFound } from "@tanstack/react-router";

import { config } from "@flow/config";
import Dev from "@flow/features/Dev";

export const Route = createFileRoute("/dev")({
  component: DevRoute,
  loader: async () => {
    const { devMode } = config();
    if (!devMode) {
      throw notFound();
    }
    return { devMode };
  },
});

function DevRoute() {
  return <Dev />;
}
