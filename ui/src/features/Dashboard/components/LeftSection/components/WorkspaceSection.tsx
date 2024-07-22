import { PlugsConnected, Plus, Toolbox, UsersThree } from "@phosphor-icons/react";
import { useNavigate } from "@tanstack/react-router";

import { Button } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { useDialogType } from "@flow/stores";

const WorkspaceSection: React.FC = () => {
  const t = useT();
  const navigate = useNavigate();
  const [, setDialogType] = useDialogType();
  return (
    <div>
      <div className="flex items-center justify-between gap-2 border-b border-zinc-700 p-2">
        <p className="font-extralight">{t("Workspace")}</p>
        <Button
          className="flex h-[30px] gap-2 bg-background-800 text-zinc-300 hover:bg-background-700 hover:text-zinc-300"
          variant="outline"
          size="sm"
          onClick={() => setDialogType("add-workspace")}>
          <Plus weight="thin" />
          <p className="text-xs font-light">{t("New Workspace")}</p>
        </Button>
      </div>
      <div className="flex flex-col gap-1 p-2">
        <div
          className="flex flex-1 cursor-pointer items-center gap-2 rounded border-zinc-700 px-2 py-1 hover:bg-background-700"
          onClick={() => navigate({ to: `settings/general` })}>
          <Toolbox weight="thin" />
          <p className="text-sm font-extralight">{t("General Settings")}</p>
        </div>
        <div
          className="flex flex-1 cursor-pointer items-center gap-2 rounded border-zinc-700 px-2 py-1 hover:bg-background-700"
          onClick={() => navigate({ to: `settings/members` })}>
          <UsersThree weight="thin" />
          <p className="text-sm font-extralight">{t("Member Settings")}</p>
        </div>
        <div
          className="flex flex-1 cursor-pointer items-center gap-2 rounded border-zinc-700 px-2 py-1 hover:bg-background-700"
          onClick={() => navigate({ to: `settings/integrations` })}>
          <PlugsConnected weight="thin" />
          <p className="text-sm font-extralight">{t("Integration Settings")}</p>
        </div>
      </div>
    </div>
  );
};

export { WorkspaceSection };
