import type { RouteOption } from "@flow/features/WorkspaceLeftPanel";

import {
  DeploymentManager,
  ProjectManager,
  WorkspaceSettings,
} from "./components";
import { TriggerManager } from "./components/TriggerManager";

type Props = {
  route?: RouteOption;
};

const EndSection: React.FC<Props> = ({ route }) => {
  console.log("selected", route);
  return (
    <div className="flex flex-1 items-end">
      <div className="flex flex-1 flex-col gap-4 p-4">
        <ProjectManager selected={route === "projects"} />
        <DeploymentManager selected={route === "deployments"} />
        <TriggerManager selected={route === "triggers"} />
        <WorkspaceSettings selected={route} />
      </div>
    </div>
  );
};

export { EndSection };
