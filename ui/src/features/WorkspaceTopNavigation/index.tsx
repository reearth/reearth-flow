import { useNavigate } from "@tanstack/react-router";

import { FlowLogo } from "@flow/components";
import { config } from "@flow/config";
import { useCurrentWorkspace } from "@flow/stores";

import { UserNavigation, WorkspaceNavigation } from "./components";

const TopNavigation: React.FC = () => {
  const { brandName } = config();
  const [currentWorkspace] = useCurrentWorkspace();

  const navigate = useNavigate();

  return (
    <div className="border-b bg-secondary">
      <div className="relative flex h-14 items-center justify-between gap-4 px-4">
        <div className="flex items-center gap-2">
          <div
            className="z-10 mr-2 cursor-pointer"
            onClick={() =>
              navigate({ to: `/workspaces/${currentWorkspace?.id}` })
            }>
            <FlowLogo className="size-8" />
          </div>
          <h1 className="select-none dark:font-extralight">
            {brandName ?? "Flow"}
          </h1>
        </div>
        <div
          id="dashboard-middle"
          className="absolute inset-x-0 flex justify-center">
          <div className="flex max-w-[40vw] justify-center gap-4">
            <WorkspaceNavigation />
          </div>
        </div>
        <div id="dashboard-right" className="z-10 flex items-center gap-5">
          <UserNavigation />
        </div>
      </div>
    </div>
  );
};

export { TopNavigation };
