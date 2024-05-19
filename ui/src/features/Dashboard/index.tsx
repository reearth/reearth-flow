import LeftPanel from "../LeftPanel";

import { LeftSection, MainSection, Nav } from "./components";

const Dashboard: React.FC = () => {
  return (
    <div className="flex flex-1 relative h-screen">
      <LeftPanel />
      <div className="[&>*]:dark flex flex-col bg-zinc-800 text-zinc-300 h-[100vh]">
        <Nav className="mt-[8px] mx-[8px]" />
        <div className="flex-1 m-[8px] flex gap-[8px]">
          <LeftSection />
          <MainSection />
        </div>
      </div>
    </div>
  );
};

export { Dashboard };
