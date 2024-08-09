import { PlugsConnected, Toolbox, UsersThree } from "@phosphor-icons/react";
import { useNavigate } from "@tanstack/react-router";

import { useT } from "@flow/lib/i18n";

const WorkspaceSection: React.FC = () => {
  const t = useT();
  const navigate = useNavigate();
  return (
    <div className="flex flex-1 items-end p-4">
      <div className="flex flex-col gap-1">
        <div
          className="flex flex-1 cursor-pointer items-center gap-2 rounded py-1 hover:bg-accent"
          onClick={() => navigate({ to: `settings/general` })}
        >
          <Toolbox weight="thin" />
          <p className="text-sm font-extralight">{t("General Settings")}</p>
        </div>
        <div
          className="flex flex-1 cursor-pointer items-center gap-2 rounded py-1 hover:bg-accent"
          onClick={() => navigate({ to: `settings/members` })}
        >
          <UsersThree weight="thin" />
          <p className="text-sm font-extralight">{t("Member Settings")}</p>
        </div>
        <div
          className="flex flex-1 cursor-pointer items-center gap-2 rounded py-1 hover:bg-accent"
          onClick={() => navigate({ to: `settings/integrations` })}
        >
          <PlugsConnected weight="thin" />
          <p className="text-sm font-extralight">{t("Integration Settings")}</p>
        </div>
      </div>
    </div>
  );
};

export { WorkspaceSection };
