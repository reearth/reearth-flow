import { createFileRoute } from "@tanstack/react-router";

import { IndexLoadingPage } from "@flow/pages";

export const Route = createFileRoute("/")({
  component: Index,
});

function Index() {
  return <IndexLoadingPage />;
}
