import { PlugsConnected, Toolbox, UsersThree } from "@phosphor-icons/react";
import { useNavigate } from "@tanstack/react-router";

import { useT } from "@flow/lib/i18n";

const WorkspaceSettings: React.FC = () => {
  const t = useT();
  const navigate = useNavigate();
  return (
    <div className="flex w-full flex-col gap-1">
      <p className="text-sm dark:font-thin">{t("Workspace")}</p>
      <div
        className="-mx-2 flex flex-1 cursor-pointer items-center gap-2 rounded px-2 py-1 hover:bg-accent"
        onClick={() => navigate({ to: `settings/general` })}>
        <Toolbox weight="light" />
        <p className="dark:font-extralight">{t("General Settings")}</p>
      </div>
      <div
        className="-mx-2 flex flex-1 cursor-pointer items-center gap-2 rounded px-2 py-1 hover:bg-accent"
        onClick={() => navigate({ to: `settings/members` })}>
        <UsersThree weight="light" />
        <p className="dark:font-extralight">{t("Member Settings")}</p>
      </div>
      <div
        className="-mx-2 flex flex-1 cursor-pointer items-center gap-2 rounded px-2 py-1 hover:bg-accent"
        onClick={() => navigate({ to: `settings/integrations` })}>
        <PlugsConnected weight="light" />
        <p className="dark:font-extralight">{t("Integration Settings")}</p>
      </div>
    </div>
  );
};

export { WorkspaceSettings };
