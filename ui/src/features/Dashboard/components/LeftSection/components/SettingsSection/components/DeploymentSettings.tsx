import { RocketLaunch } from "@phosphor-icons/react";
import { useNavigate } from "@tanstack/react-router";

import { useT } from "@flow/lib/i18n";

const DeploymentSettings: React.FC = () => {
  const t = useT();
  const navigate = useNavigate();
  return (
    <div className="flex w-full flex-col gap-1">
      <p className="text-sm dark:font-thin">{t("Deployments")}</p>
      <div
        className="-mx-2 flex flex-1 cursor-pointer items-center gap-2 rounded px-2 py-1 hover:bg-accent"
        onClick={() => navigate({ to: `settings/deployments` })}>
        <RocketLaunch weight="light" />
        <p className="dark:font-extralight">{t("Manage deployments")}</p>
      </div>
    </div>
  );
};

export { DeploymentSettings };
