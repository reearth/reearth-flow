import { useParams } from "@tanstack/react-router";

import { Loading } from "@flow/components";
import { useCheckWorkspace } from "@flow/utils/router/checkWorkspace";

import { TopNavigation } from "../TopNavigation";

import { LeftSection, MainSection } from "./components";

const Dashboard: React.FC = () => {
  const { workspaceId } = useParams({ strict: false });

  const { workspaces, isLoading } = useCheckWorkspace(workspaceId);

  if (isLoading) return <Loading />;

  // TODO: Show proper error
  if (!workspaces) return <div>Could not fetch workspaces</div>;

  return (
    <div className="[&>*]:dark flex flex-col bg-zinc-800 text-zinc-300 h-[100vh]">
      <TopNavigation />
      <div className="flex-1 flex">
        <LeftSection />
        <MainSection />
      </div>
    </div>
  );
};

export { Dashboard };
