import { FlowLogo, NavigationMenu, NavigationMenuList } from "@flow/components";
import { config } from "@flow/config";
import { useT } from "@flow/providers";

import { UserNavigation, WorkspaceNavigation } from "./components";

type Props = {
  className?: string;
};

const Nav: React.FC<Props> = ({ className }) => {
  const t = useT();
  const { brandName, version } = config();
  return (
    <div className={`bg-zinc-900/50 border border-zinc-700 rounded-lg ${className}`}>
      <div className="relative flex justify-between items-center gap-4 h-14 px-4">
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
  );
};

export { Nav };
