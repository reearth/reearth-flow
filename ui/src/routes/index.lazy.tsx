import { createLazyFileRoute } from "@tanstack/react-router";

import { LoadingPage } from "@flow/features/LoadingPage";

export const Route = createLazyFileRoute("/")({
  component: Index,
});

function Index() {
  return <LoadingPage />;
}
