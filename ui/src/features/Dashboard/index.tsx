import { FlowLogo, NavigationMenu, NavigationMenuList } from "@flow/components";
import { config } from "@flow/config";
import { useT } from "@flow/providers";

import { LeftSection, MainSection } from "./components";
import { WorkspaceNavigation } from "./components/LeftSection/components";
import { UserNavigation } from "./components/UserNavigation";

const Dashboard: React.FC = () => {
  const t = useT();
  const { brandName, version } = config();

  return (
    <div className="[&>*]:dark relative bg-zinc-800 pt-14 text-zinc-300 h-[100vh]">
      <div className="absolute left-0 right-0 top-0">
        <div className="relative flex justify-between items-center gap-4 h-14 px-4 bg-zinc-900/50">
          <div className="flex gap-2 items-center">
            <div className="flex bg-red-800/50 p-2 rounded">
              <FlowLogo />
            </div>
            <h1 className="text-md font-extralight select-none">
              {brandName ?? t("Re:Earth Flow")} {version ?? "X.X.X"}
            </h1>
          </div>
          <div id="dashboard-middle" className="absolute left-0 right-0 flex justify-center">
            <WorkspaceNavigation className="max-w-[40vw]" />
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
      <div className="h-[calc(100%-16px)] m-[8px] flex gap-[8px]">
        <LeftSection />
        <MainSection />
      </div>
    </div>
  );
};

export { Dashboard };
