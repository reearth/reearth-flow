import { Loading } from "@flow/components";
import { TopNavigation } from "@flow/features/TopNavigation";
import { useCurrentWorkspace } from "@flow/stores";

import { LeftSection, MainSection } from "./components";

const Dashboard: React.FC = () => {
  const [currentWorkspace] = useCurrentWorkspace();

  return currentWorkspace ? (
    <div className="flex h-screen flex-col bg-background-800 text-zinc-300">
      <TopNavigation />
      <div className="flex flex-1">
        <LeftSection />
        <MainSection workspace={currentWorkspace} />
      </div>
    </div>
  ) : (
    <Loading />
  );
};

export { Dashboard };
