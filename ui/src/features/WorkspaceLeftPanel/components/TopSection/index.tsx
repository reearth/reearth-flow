import { useNavigate } from "@tanstack/react-router";

import { EcosystemNavigator, FlowLogo } from "@flow/components";
import { config } from "@flow/config";
import { UserMenu, WorkspaceMenu } from "@flow/features/common";
import type { RouteOption } from "@flow/features/WorkspaceLeftPanel";
import { useCurrentProject, useCurrentWorkspace } from "@flow/stores";

import {
  DeploymentManager,
  ProjectManager,
  JobManager,
  TriggerManager,
  AssetManager,
} from "./components";

type Props = {
  route?: RouteOption;
};

const TopSection: React.FC<Props> = ({ route }) => {
  const { brandName } = config();
  const [currentWorkspace] = useCurrentWorkspace();
  const [, setCurrentProject] = useCurrentProject();

  const navigate = useNavigate();
  return (
    <div className="flex flex-1 flex-col gap-2">
      <div className="flex flex-col">
        <div
          className="flex cursor-pointer items-center justify-between gap-2 px-1 pt-4 pb-1"
          onClick={() => {
            setCurrentProject(undefined);
            navigate({ to: `/workspaces/${currentWorkspace?.id}/projects` });
          }}>
          <div className="flex items-center gap-2">
            <FlowLogo className="size-8" />
            <p className="select-none dark:font-thin">{brandName ?? "Flow"}</p>
          </div>
          {/* Stops clicks (including ones inside the portalled popovers) from
              reaching the row's navigate handler */}
          <div
            className="flex items-center gap-2"
            onClick={(e) => e.stopPropagation()}>
            <EcosystemNavigator />
            <UserMenu dropdownAlign="center" dropdownPosition="bottom" />
          </div>
        </div>
      </div>
      <WorkspaceMenu />
      <div className="h-px bg-border" />
      <div className="flex flex-col gap-2 px-2">
        <ProjectManager selected={route === "projects"} />
        <DeploymentManager selected={route === "deployments"} />
        <TriggerManager selected={route === "triggers"} />
        <JobManager selected={route === "jobs"} />
      </div>
      <div className="h-px bg-border" />
      <div className="flex flex-1 flex-col gap-2 px-2">
        <AssetManager selected={route === "assets"} />
      </div>
    </div>
  );
};

export { TopSection };
