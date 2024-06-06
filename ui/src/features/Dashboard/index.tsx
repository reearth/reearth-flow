import { TopNavigation } from "../TopNavigation";

import { LeftSection, MainSection } from "./components";

const Dashboard: React.FC = () => {
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
