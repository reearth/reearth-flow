import {
  ArrowSquareOutIcon,
  ToolboxIcon,
  UsersThreeIcon,
} from "@phosphor-icons/react";
import { useNavigate } from "@tanstack/react-router";

import type { RouteOption } from "@flow/features/WorkspaceLeftPanel";
import { useT } from "@flow/lib/i18n";
import { useCurrentWorkspace } from "@flow/stores";
import {
  dashboardWorkspaceMembersUrl,
  dashboardWorkspaceSettingsUrl,
  openLinkInNewTab,
} from "@flow/utils";

type Props = {
  selected?: RouteOption;
};

const WorkspaceSettings: React.FC<Props> = ({ selected }) => {
  const t = useT();
  const navigate = useNavigate();
  const [currentWorkspace] = useCurrentWorkspace();

  // With a Dashboard, workspace settings and members are managed there.
  const alias = currentWorkspace?.alias;
  const externalSettingsUrl = alias
    ? dashboardWorkspaceSettingsUrl(alias)
    : undefined;
  const externalMembersUrl = alias
    ? dashboardWorkspaceMembersUrl(alias)
    : undefined;

  return (
    <div className="flex w-full flex-col gap-1">
      <div
        className={`-mx-2 flex flex-1 cursor-pointer items-center gap-2 rounded px-2 py-1 ${!externalSettingsUrl && selected === "general" && "bg-accent"} hover:bg-accent`}
        onClick={
          externalSettingsUrl
            ? openLinkInNewTab(externalSettingsUrl)
            : () =>
                navigate({
                  to: `/workspaces/${currentWorkspace?.id}/settings/general`,
                })
        }>
        <ToolboxIcon weight="light" />
        <p className="text-sm dark:font-extralight">{t("General Settings")}</p>
        {externalSettingsUrl && <ArrowSquareOutIcon weight="light" />}
      </div>
      {/* A personal workspace has no members to manage in the Dashboard. */}
      {!(externalMembersUrl && currentWorkspace?.personal) && (
        <div
          className={`-mx-2 flex flex-1 cursor-pointer items-center gap-2 rounded px-2 py-1 ${!externalMembersUrl && selected === "members" && "bg-accent"} hover:bg-accent`}
          onClick={
            externalMembersUrl
              ? openLinkInNewTab(externalMembersUrl)
              : () =>
                  navigate({
                    to: `/workspaces/${currentWorkspace?.id}/settings/members`,
                  })
          }>
          <UsersThreeIcon weight="light" />
          <p className="text-sm dark:font-extralight">{t("Member Settings")}</p>
          {externalMembersUrl && <ArrowSquareOutIcon weight="light" />}
        </div>
      )}
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
