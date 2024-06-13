import { createLazyFileRoute } from "@tanstack/react-router";

import { Editor } from "@flow/pages";

export const Route = createLazyFileRoute("/workspace/$workspaceId/project/$projectId")({
  component: Editor,
});
