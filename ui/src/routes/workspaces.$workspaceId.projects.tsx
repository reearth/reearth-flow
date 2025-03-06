import { createFileRoute } from "@tanstack/react-router";

import ProjectsManager from "@flow/features/WorkspaceProjects";

export const Route = createFileRoute("/workspaces/$workspaceId/projects")({
  component: () => <ProjectsManager />,
});
