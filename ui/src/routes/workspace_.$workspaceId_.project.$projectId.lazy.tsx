import { createLazyFileRoute } from "@tanstack/react-router";

import Canvas from "@flow/features/Editor";
import { useCurrentProject } from "@flow/stores";

export const Route = createLazyFileRoute("/workspace/$workspaceId/project/$projectId")({
  component: Editor,
});

function Editor() {
  // TODO: Update this once the PROJECT CRUD PR is merged
  const [currentProject] = useCurrentProject();

  return (
    <div className="flex flex-col bg-zinc-900 text-zinc-300 h-screen">
      <Canvas workflow={currentProject?.workflow} />
    </div>
  );
}
