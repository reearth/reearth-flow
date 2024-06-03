import { Plus, UsersThree } from "@phosphor-icons/react";

import { ButtonWithTooltip, FlowLogo } from "@flow/components";
import { config } from "@flow/config";
import { useT } from "@flow/providers";
import { useDialogType } from "@flow/stores";

import { UserNavigation, WorkspaceNavigation } from "./components";

type Props = {
  className?: string;
};

const Nav: React.FC<Props> = ({ className }) => {
  const t = useT();
  const { brandName, version } = config();
  const [, setDialogType] = useDialogType();

  return (
    <div className={`bg-zinc-900/50 border-b border-zinc-700 ${className}`}>
      <div className="relative flex justify-between items-center gap-4 h-14 px-4">
        <div className="flex gap-2 items-center">
          <div className="bg-red-800/50 p-2 rounded">
            <FlowLogo className="h-5 w-5" />
          </div>
          <h1 className="text-md font-extralight select-none">
            {brandName ?? t("Re:Earth Flow")} {version ?? "X.X.X"}
          </h1>
        </div>
        <div id="dashboard-middle" className="absolute left-0 right-0 flex justify-center">
          <div className="flex justify-center gap-4 max-w-[40vw]">
            <WorkspaceNavigation />
            <ButtonWithTooltip
              className="flex bg-zinc-800 text-zinc-300 hover:bg-zinc-700 hover:text-zinc-300"
              variant="outline"
              onClick={() => setDialogType("add-workspace")}
              tooltipText={t("Create new workspace")}>
              <UsersThree className="w-5 h-5" weight="thin" />
              <Plus className="w-2 h-2" />
            </ButtonWithTooltip>
          </div>
        </div>
        <div id="dashboard-right" className="flex items-center z-10">
          <UserNavigation />
        </div>
      </div>
    </div>
  );
};

export { Nav };
