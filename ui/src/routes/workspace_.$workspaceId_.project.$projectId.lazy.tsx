import { createLazyFileRoute, useNavigate } from "@tanstack/react-router";
import { useEffect } from "react";

import BottomPanel from "@flow/features/BottomPanel";
import Canvas from "@flow/features/Canvas";
import LeftPanel from "@flow/features/LeftPanel";
import { workspaces } from "@flow/mock_data/workspaceData";
import { useCurrentProject, useCurrentWorkspace } from "@flow/stores";

export const Route = createLazyFileRoute("/workspace/$workspaceId/project/$projectId")({
  component: Editor,
});

function Editor() {
  const [currentWorkspace, setCurrentWorkspace] = useCurrentWorkspace();
  const [currentProject] = useCurrentProject();
  const navigate = useNavigate({ from: "workspace/$workspaceId/project/$projectId" });

  useEffect(() => {
    if (currentWorkspace && !currentProject) {
      navigate({ to: "/workspace" });
    }
  }, [currentWorkspace, currentProject, navigate]);

  useEffect(() => {
    if (!currentWorkspace) {
      setCurrentWorkspace(workspaces[0]);
    }
  }, [currentWorkspace, setCurrentWorkspace]);

  return (
    <div className="flex flex-col bg-zinc-900 text-zinc-300 h-screen">
      <div className="flex flex-1">
        <div className="flex flex-col flex-1 p-0">
          <Canvas
            workflow={currentProject?.workflows?.[0]}
            leftArea={<LeftPanel data={currentProject?.workflows?.[0]} />}
          />
          <BottomPanel />
        </div>
      </div>
    </div>
  );
}
