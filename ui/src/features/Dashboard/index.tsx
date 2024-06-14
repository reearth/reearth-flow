import { ErrorPage } from "@flow/features/ErrorPage";
import { TopNavigation } from "@flow/features/TopNavigation";
import { useCurrentWorkspace } from "@flow/stores";

import { LeftSection, MainSection } from "./components";

const Dashboard: React.FC = () => {
  const [currentWorkspace] = useCurrentWorkspace();

  return currentWorkspace ? (
    <div className="[&>*]:dark flex flex-col bg-zinc-800 text-zinc-300 h-[100vh]">
      <TopNavigation />
      <div className="flex-1 flex">
        <LeftSection />
        <MainSection workspace={currentWorkspace} />
      </div>
    </div>
  ) : (
    <ErrorPage errorMessage={"Workspace not set"} />
  );
};

export { Dashboard };
