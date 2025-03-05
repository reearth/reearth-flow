import { useNavigate } from "@tanstack/react-router";

import { FlowLogo } from "@flow/components";
import { config } from "@flow/config";
import { UserMenu, WorkspaceMenu } from "@flow/features/common";
import type { RouteOption } from "@flow/features/WorkspaceLeftPanel";
import { useCurrentWorkspace } from "@flow/stores";

import { DeploymentManager, ProjectManager, JobManager } from "./components";
import { TriggerManager } from "./components/TriggerManager";

type Props = {
  route?: RouteOption;
};

const TopSection: React.FC<Props> = ({ route }) => {
  const { brandName } = config();
  const [currentWorkspace] = useCurrentWorkspace();

  const navigate = useNavigate();
  return (
    <div className="flex flex-1 flex-col gap-2">
      <div className="flex flex-col">
        <div
          className="flex cursor-pointer items-center justify-between gap-2 p-4"
          onClick={() =>
            navigate({ to: `/workspaces/${currentWorkspace?.id}/projects` })
          }>
          <div className="flex items-center gap-2">
            <FlowLogo className="size-8" />
            <h1 className="select-none font-light">{brandName ?? "Flow"}</h1>
          </div>
          <UserMenu dropdownAlign="center" dropdownPosition="bottom" />
        </div>
        <div className="h-px bg-primary" />
      </div>
      <WorkspaceMenu />
      <div className="flex flex-1 flex-col gap-2 px-4">
        <ProjectManager selected={route === "projects"} />
        <DeploymentManager selected={route === "deployments"} />
        <TriggerManager selected={route === "triggers"} />
        <JobManager selected={route === "jobs"} />
      </div>
    </div>
  );
};

export { TopSection };
