import { Loading } from "@flow/components";
import { DeploymentManager } from "@flow/features/DeploymentsManager";
import { TopNavigation } from "@flow/features/TopNavigation";
import { useCurrentWorkspace } from "@flow/stores";

import { LeftSection, MainSection } from "./components";

type Props = {
  baseRoute?: "deployments" | "projects";
};

const Dashboard: React.FC<Props> = ({ baseRoute }) => {
  const [currentWorkspace] = useCurrentWorkspace();

  return currentWorkspace ? (
    <div className="flex h-screen flex-col">
      <TopNavigation />
      <div className="flex h-[calc(100vh-57px)] flex-1">
        <LeftSection baseRoute={baseRoute} workspaceId={currentWorkspace.id} />
        {baseRoute === "deployments" ? (
          <DeploymentManager workspace={currentWorkspace} />
        ) : (
          <MainSection workspace={currentWorkspace} />
        )}
      </div>
    </div>
  ) : (
    <Loading />
  );
};

export { Dashboard };
