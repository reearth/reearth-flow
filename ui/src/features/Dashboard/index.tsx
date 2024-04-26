import { NavigationMenu, NavigationMenuList } from "@flow/components";

import { LeftSection, MainSection } from "./components";
import { UserNavigation } from "./components/UserNavigation";

const Dashboard: React.FC = () => {
  return (
    <div className="[&>*]:dark relative bg-zinc-800 pt-20 text-zinc-300 h-[100vh]">
      <div className="absolute left-0 right-0 top-0">
        <div className="relative flex justify-end items-center gap-4 h-20 px-4">
          <div id="dashboard-middle" className="absolute left-0 right-0 flex justify-center">
            <h1 className="text-lg font-thin">Re:Earth Flow</h1>
          </div>
          <div id="dashboard-right" className="z-10">
            <NavigationMenu>
              <NavigationMenuList>
                <UserNavigation />
              </NavigationMenuList>
            </NavigationMenu>
          </div>
        </div>
      </div>
      <div className="border-t border-zinc-700 w-full" />
      <div className="h-[calc(100%-8px)] m-2 flex">
        <LeftSection />
        <MainSection />
      </div>
    </div>
  );
};

export default Dashboard;
