import type { RouteOption } from "@flow/features/WorkspaceLeftPanel";

import {
  DeploymentManager,
  ProjectManager,
  WorkspaceSettings,
} from "./components";

type Props = {
  route?: RouteOption;
};

const EndSection: React.FC<Props> = ({ route }) => {
  return (
    <div className="flex flex-1 items-end">
      <div className="flex flex-1 flex-col gap-4 p-4">
        <ProjectManager selected={route === "projects"} />
        <DeploymentManager selected={route === "deployments"} />
        <WorkspaceSettings selected={route} />
      </div>
    </div>
  );
};

export { EndSection };
