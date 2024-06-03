import { PlugsConnected, Plus, Toolbox, UsersThree } from "@phosphor-icons/react";
import { useNavigate } from "@tanstack/react-router";

import { Button } from "@flow/components";
import { useT } from "@flow/providers";

const WorkspaceSection: React.FC = () => {
  const t = useT();
  const navigate = useNavigate();
  return (
    <div>
      <div className="flex gap-2 justify-between items-center border-b border-zinc-700 py-2 px-2">
        <p className="font-extralight">{t("Workspace")}</p>
        <Button
          className="flex gap-2 h-[30px] bg-zinc-800 text-zinc-300 hover:bg-zinc-700 hover:text-zinc-300"
          variant="outline"
          size="sm">
          <Plus weight="thin" />
          <p className="text-xs font-light">{t("New Workspace")}</p>
        </Button>
      </div>
      <div className="flex flex-col gap-1 p-2">
        <div
          className="flex flex-1 gap-2 items-center border-zinc-700 rounded px-2 py-1 cursor-pointer hover:bg-zinc-700"
          onClick={() => navigate({ to: `settings/general` })}>
          <Toolbox weight="thin" />
          <p className="text-sm font-extralight">{t("General Settings")}</p>
        </div>
        <div
          className="flex flex-1 gap-2 items-center border-zinc-700 rounded px-2 py-1 cursor-pointer hover:bg-zinc-700"
          onClick={() => navigate({ to: `settings/members` })}>
          <UsersThree weight="thin" />
          <p className="text-sm font-extralight">{t("Member Settings")}</p>
        </div>
        <div
          className="flex flex-1 gap-2 items-center border-zinc-700 rounded px-2 py-1 cursor-pointer hover:bg-zinc-700"
          onClick={() => navigate({ to: `settings/integrations` })}>
          <PlugsConnected weight="thin" />
          <p className="text-sm font-extralight">{t("Integration Settings")}</p>
        </div>
      </div>
    </div>
  );
};

export { WorkspaceSection };
