import { useNavigate } from "@tanstack/react-router";

import { FlowLogo } from "@flow/components";
import { config } from "@flow/config";
import { useCurrentWorkspace } from "@flow/stores";

import { UserNavigation, WorkspaceNavigation } from "../Dashboard/components/Nav/components";

const TopNavigation: React.FC = () => {
  const { brandName, version } = config();
  const [currentWorkspace] = useCurrentWorkspace();

  const navigate = useNavigate();

  return (
    <div className={`bg-zinc-900/50 border-b border-zinc-700`}>
      <div className="relative flex justify-between items-center gap-4 h-14 px-4">
        <div className="flex gap-2 items-center">
          <div
            className="bg-red-800/50 p-2 rounded cursor-pointer z-10"
            onClick={() => navigate({ to: `/workspace/${currentWorkspace?.id}` })}>
            <FlowLogo className="h-5 w-5" />
          </div>
          <h1 className="text-md font-extralight select-none">
            {brandName ?? "Re:Earth Flow"} {version ?? "X.X.X"}
          </h1>
        </div>
        <div id="dashboard-middle" className="absolute left-0 right-0 flex justify-center">
          <div className="flex justify-center gap-4 max-w-[40vw]">
            <WorkspaceNavigation />
          </div>
        </div>
        <div id="dashboard-right" className="flex items-center z-10">
          <UserNavigation />
        </div>
      </div>
    </div>
  );
};

export { TopNavigation };
