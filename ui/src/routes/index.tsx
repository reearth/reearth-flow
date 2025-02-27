import { createFileRoute, redirect } from "@tanstack/react-router";

import { LoadingSplashscreen } from "@flow/components";

export const Route = createFileRoute("/")({
  component: () => <LoadingSplashscreen />,
  loader: () => {
    throw redirect({ to: "/workspaces" });
  },
});
