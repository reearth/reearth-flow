import { createFileRoute, redirect } from "@tanstack/react-router";

import { LoadingSplashscreen } from "@flow/components";
import ErrorPage from "@flow/components/errors/ErrorPage";

export const Route = createFileRoute("/")({
  component: () => <LoadingSplashscreen />,
  errorComponent: () => <ErrorPage errorMessage={"Something Went Wrong"} />,
  loader: () => {
    throw redirect({ to: "/workspaces" });
  },
});
