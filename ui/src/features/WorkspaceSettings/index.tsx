import { ToolboxIcon, UsersThreeIcon } from "@phosphor-icons/react";
import { Navigate, useParams, useRouterState } from "@tanstack/react-router";

import { useT } from "@flow/lib/i18n";
import { usePermissions } from "@flow/lib/permissions";

import type { RouteOption } from "../WorkspaceLeftPanel";

import { GeneralSettings, MembersSettings } from "./components";

type Tab = "general" | "integrations" | "members";

const WorkspaceSettings: React.FC = () => {
  const t = useT();
  const { viewMembers } = usePermissions();
  const { workspaceId } = useParams({ strict: false });
  const {
    location: { pathname },
  } = useRouterState();

  const selectedTab: RouteOption = pathname.includes("integrations")
    ? "integrations"
    : pathname.includes("members")
      ? "members"
      : "general";

  const content: {
    id: Tab;
    name: string;
    icon: React.ReactNode;
    component: React.ReactNode;
  }[] = [
    {
      id: "general",
      name: t("General"),
      icon: <ToolboxIcon weight="light" />,
      component: <GeneralSettings />,
    },
    {
      id: "members",
      name: t("Members"),
      icon: <UsersThreeIcon weight="light" />,
      component: <MembersSettings />,
    },
  ];

  if (!viewMembers) {
    return (
      <Navigate
        to="/workspaces/$workspaceId/projects"
        params={{ workspaceId: workspaceId ?? "" }}
        replace
      />
    );
  }

  return (
    <div className="flex h-full flex-1 flex-col">
      <div className="flex flex-1 flex-col pt-4 pr-3 pb-2 pl-2">
        {content.find((c) => c.id === selectedTab)?.component}
      </div>
    </div>
  );
};

export default WorkspaceSettings;
