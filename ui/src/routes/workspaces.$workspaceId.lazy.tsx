import { createLazyFileRoute } from "@tanstack/react-router";

import ProjectsManager from "@flow/features/WorkspaceProjects";

export const Route = createLazyFileRoute("/workspaces/$workspaceId")({
  component: () => <ProjectsManager />,
});
