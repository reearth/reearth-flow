import { createRootRoute, Outlet, useParams } from "@tanstack/react-router";
import { TanStackRouterDevtools } from "@tanstack/router-devtools";
import { ReactFlowProvider } from "reactflow";

import Dialog from "@flow/features/Dialog";
import { workspaces } from "@flow/mock_data/workspaceData";
import { I18nProvider, TooltipProvider } from "@flow/providers";
import { useCurrentProject, useCurrentWorkspace } from "@flow/stores";

const RootRoute: React.FC = () => {
  const [currentProject, setCurrentProject] = useCurrentProject();
  const [currentWorkspace, setCurrentWorkspace] = useCurrentWorkspace();

  const { projectId, workspaceId } = useParams({ strict: false });

  if ((workspaceId && !currentWorkspace) || workspaceId !== currentWorkspace?.id) {
    setCurrentWorkspace(workspaces.find(w => w.id === workspaceId));
  }

  if (currentWorkspace && ((projectId && !currentProject) || projectId !== currentProject?.id)) {
    setCurrentProject(currentWorkspace.projects?.find(p => p.id === projectId));
  }

  return (
    <>
      <I18nProvider>
        <TooltipProvider>
          <ReactFlowProvider>
            <Dialog />
            <Outlet />
          </ReactFlowProvider>
        </TooltipProvider>
      </I18nProvider>
      <TanStackRouterDevtools position="bottom-right" />
    </>
  );
};

export const Route = createRootRoute({
  component: RootRoute,
});
