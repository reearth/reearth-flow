import { useNavigate } from "@tanstack/react-router";
import { useEffect } from "react";

import { FlowLogo, NavigationMenu, NavigationMenuList } from "@flow/components";
import { config } from "@flow/config";
import { workspaces } from "@flow/mock_data/workspaceData";
import { useT } from "@flow/providers";
import { useCurrentWorkspace } from "@flow/stores";

import { LeftSection, MainSection } from "./components";
import { WorkspaceNavigation } from "./components/LeftSection/components";
import { UserNavigation } from "./components/UserNavigation";

const Dashboard: React.FC = () => {
  const t = useT();
  const [currentWorkspace, setCurrentWorkspace] = useCurrentWorkspace();
  const navigate = useNavigate({ from: "/workspace" });

  const workspaceIdFromUrl = new URLSearchParams(window.location.search).get("workspace") ?? "";

  console.log("WorkspaceIdFromUrl", workspaceIdFromUrl);

  useEffect(() => {
    if (!currentWorkspace && workspaceIdFromUrl) {
      setCurrentWorkspace(workspaces[0]);
      navigate({ to: `/workspace/${workspaces[0].id}` });
    }
  }, [currentWorkspace, workspaceIdFromUrl, navigate, setCurrentWorkspace]);

  return (
    <div className="[&>*]:dark relative bg-zinc-800 pt-16 text-zinc-300 h-[100vh]">
      <div className="absolute left-0 right-0 top-0">
        <div className="relative flex justify-between items-center gap-4 h-16 px-4">
          <div className="flex gap-2 items-center">
            <div className="flex bg-red-800/50 p-2 rounded">
              <FlowLogo />
            </div>
            <h1 className="text-md font-extralight select-none">
              {config()?.brandName ??
                t("Re:Earth Flow") + (config()?.version && ` ${config()?.version}`)}
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
      <div className="h-[calc(100%-9px)] m-[8px] flex">
        <LeftSection />
        <MainSection />
      </div>
    </div>
  );
};

export default Dashboard;
