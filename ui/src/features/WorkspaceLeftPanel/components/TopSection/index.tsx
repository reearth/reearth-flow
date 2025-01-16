import { useNavigate } from "@tanstack/react-router";

import { FlowLogo } from "@flow/components";
import { config } from "@flow/config";
import { UserMenu } from "@flow/features/common";
import type { RouteOption } from "@flow/features/WorkspaceLeftPanel";
import { useT } from "@flow/lib/i18n";
import { useCurrentWorkspace } from "@flow/stores";

import { DeploymentManager, ProjectManager, JobManager } from "./components";

type Props = {
  route?: RouteOption;
};

const TopSection: React.FC<Props> = ({ route }) => {
  const t = useT();
  const { brandName } = config();
  const [currentWorkspace] = useCurrentWorkspace();

  const navigate = useNavigate();
  return (
    <div className="flex flex-1 flex-col">
      <div className="flex flex-col">
        <div
          className="flex cursor-pointer items-center justify-center gap-2 p-4"
          onClick={() =>
            navigate({ to: `/workspaces/${currentWorkspace?.id}` })
          }>
          <FlowLogo className="size-8" />
          <h1 className="select-none font-light">{brandName ?? "Flow"}</h1>
        </div>
        <div className="h-px bg-primary" />
        <div className="flex flex-1 flex-col gap-2 p-4">
          <p className="text-xs dark:font-thin">{t("User")}</p>
          <UserMenu />
        </div>
      </div>
      <div className="flex flex-1 flex-col gap-2 p-4">
        <p className="text-xs dark:font-thin">{t("General")}</p>
        <ProjectManager selected={route === "projects"} />
        <DeploymentManager selected={route === "deployments"} />
        <JobManager selected={route === "jobs"} />
      </div>
    </div>
  );
};

export { TopSection };
