import { ToolboxIcon, UsersThreeIcon } from "@phosphor-icons/react";
import { useRouterState } from "@tanstack/react-router";

import { useT } from "@flow/lib/i18n";

import { RouteOption } from "../WorkspaceLeftPanel";

import { GeneralSettings, MembersSettings } from "./components";

type Tab = "general" | "integrations" | "members";

const WorkspaceSettings: React.FC = () => {
  const t = useT();
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

  return (
    <div className="flex h-full flex-1 flex-col">
      <div className="flex flex-1 flex-col gap-4 px-6 pb-2 pt-4">
        {content.find((c) => c.id === selectedTab)?.component}
      </div>
    </div>
  );
};

export default WorkspaceSettings;
