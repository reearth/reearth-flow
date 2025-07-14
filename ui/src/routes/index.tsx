import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { useEffect } from "react";

import { LoadingSplashscreen } from "@flow/components";
import ErrorPage from "@flow/components/errors/ErrorPage";
import { useAuth } from "@flow/lib/auth";

export const Route = createFileRoute("/")({
  component: Index,
  errorComponent: () => <ErrorPage />,
});

function Index() {
  const { isLoading } = useAuth();
  const navigate = useNavigate();

  useEffect(() => {
    if (isLoading) return;
    navigate({ to: "/workspaces", replace: true });
  });
  return <LoadingSplashscreen />;
}
