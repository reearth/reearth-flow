import {
  DeploymentManager,
  ProjectManager,
  WorkspaceSettings,
} from "./components";

type Props = {
  baseRoute?: "deployments" | "projects";
};

const EndSection: React.FC<Props> = ({ baseRoute }) => {
  return (
    <div className="flex flex-1 items-end">
      <div className="flex flex-1 flex-col gap-4 p-4">
        <ProjectManager selected={baseRoute === "projects"} />
        <DeploymentManager selected={baseRoute === "deployments"} />
        <WorkspaceSettings />
      </div>
    </div>
  );
};

export { EndSection };
