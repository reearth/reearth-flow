import type { RouteOption } from "@flow/features/WorkspaceLeftPanel";
import { useT } from "@flow/lib/i18n";

import { DeploymentManager, ProjectManager, JobManager } from "./components";

type Props = {
  route?: RouteOption;
};

const TopSection: React.FC<Props> = ({ route }) => {
  const t = useT();
  return (
    <div className="flex flex-1">
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
