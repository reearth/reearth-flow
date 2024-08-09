import { Loading } from "@flow/components";
import { TopNavigation } from "@flow/features/TopNavigation";
import { useCurrentWorkspace } from "@flow/stores";

import { LeftSection, MainSection } from "./components";

const Dashboard: React.FC = () => {
  const [currentWorkspace] = useCurrentWorkspace();

  return currentWorkspace ? (
    <div className="flex h-screen flex-col">
      <TopNavigation />
      <div className="flex h-[calc(100vh-57px)] flex-1">
        <LeftSection />
        <MainSection workspace={currentWorkspace} />
      </div>
    </div>
  ) : (
    <Loading />
  );
};

export { Dashboard };
