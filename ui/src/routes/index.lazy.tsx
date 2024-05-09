import { createLazyFileRoute } from "@tanstack/react-router";

import { LoadingScreen } from "@flow/features/LoadingScreen";

export const Route = createLazyFileRoute("/")({
  component: Index,
});

function Index() {
  return <LoadingScreen />;
}
