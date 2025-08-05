import {
  HardDriveIcon,
  ToolboxIcon,
  UsersThreeIcon,
} from "@phosphor-icons/react";
import { useNavigate } from "@tanstack/react-router";

import { RouteOption } from "@flow/features/WorkspaceLeftPanel";
import { useT } from "@flow/lib/i18n";
import { useCurrentWorkspace } from "@flow/stores";

type Props = {
  selected?: RouteOption;
};

const WorkspaceSettings: React.FC<Props> = ({ selected }) => {
  const t = useT();
  const navigate = useNavigate();
  const [currentWorkspace] = useCurrentWorkspace();

  return (
    <div className="flex w-full flex-col gap-1">
      <div
        className={`-mx-2 flex flex-1 cursor-pointer items-center gap-2 rounded px-2 py-1 ${selected === "assets" && "bg-accent"} hover:bg-accent`}
        onClick={() =>
          navigate({
            to: `/workspaces/${currentWorkspace?.id}/assets`,
          })
        }>
        <HardDriveIcon weight="light" />
        <p className="text-sm dark:font-extralight">{t("Assets Manager")}</p>
      </div>
      <div
        className={`-mx-2 flex flex-1 cursor-pointer items-center gap-2 rounded px-2 py-1 ${selected === "general" && "bg-accent"} hover:bg-accent`}
        onClick={() =>
          navigate({
            to: `/workspaces/${currentWorkspace?.id}/settings/general`,
          })
        }>
        <ToolboxIcon weight="light" />
        <p className="text-sm dark:font-extralight">{t("General Settings")}</p>
      </div>
      <div
        className={`-mx-2 flex flex-1 cursor-pointer items-center gap-2 rounded px-2 py-1 ${selected === "members" && "bg-accent"} hover:bg-accent`}
        onClick={() =>
          navigate({
            to: `/workspaces/${currentWorkspace?.id}/settings/members`,
          })
        }>
        <UsersThreeIcon weight="light" />
        <p className="text-sm dark:font-extralight">{t("Member Settings")}</p>
      </div>
      {/* <div
        className={`-mx-2 flex flex-1 cursor-pointer items-center gap-2 rounded px-2 py-1 ${selected === "integrations" && "bg-accent"} hover:bg-accent`}
        onClick={() =>
          navigate({
            to: `/workspaces/${currentWorkspace?.id}/settings/integrations`,
          })
        }>
        <PlugsConnected weight="light" />
        <p className="dark:font-extralight">{t("Integration Settings")}</p>
      </div> */}
    </div>
  );
};

export { WorkspaceSettings };
