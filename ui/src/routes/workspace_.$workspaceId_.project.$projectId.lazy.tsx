import { createLazyFileRoute } from "@tanstack/react-router";
import { ReactFlowProvider } from "reactflow";

import Canvas from "@flow/features/Editor";
import { useCurrentProject } from "@flow/stores";

export const Route = createLazyFileRoute("/workspace/$workspaceId/project/$projectId")({
  component: Editor,
});

function Editor() {
  const [currentProject] = useCurrentProject();

  return (
    <div className="flex flex-col bg-zinc-900 text-zinc-300 h-screen">
      <ReactFlowProvider>
        <Canvas workflow={currentProject?.workflow} />
      </ReactFlowProvider>
    </div>
  );
}
