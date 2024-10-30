import {
  DeploymentManager,
  ProjectManager,
  WorkspaceSettings,
} from "./components";

type Props = {
  baseRoute?: "deployments" | "projects";
  workspaceId: string;
};

const EndSection: React.FC<Props> = ({ baseRoute, workspaceId }) => {
  return (
    <div className="flex flex-1 items-end">
      <div className="flex flex-1 flex-col gap-4 p-4">
        <ProjectManager selected={baseRoute === "projects"} />
        <DeploymentManager selected={baseRoute === "deployments"} />
        <WorkspaceSettings workspaceId={workspaceId} />
      </div>
    </div>
  );
};

export { EndSection };
