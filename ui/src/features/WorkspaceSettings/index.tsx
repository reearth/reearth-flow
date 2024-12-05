import { Toolbox, UsersThree } from "@phosphor-icons/react";
import { useRouterState } from "@tanstack/react-router";

import { useT } from "@flow/lib/i18n";

import { RouteOption } from "../WorkspaceLeftPanel";

import {
  GeneralSettings,
  // IntegrationsSettings,
  MembersSettings,
} from "./components";

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
      icon: <Toolbox weight="light" />,
      component: <GeneralSettings />,
    },
    {
      id: "members",
      name: t("Members"),
      icon: <UsersThree weight="light" />,
      component: <MembersSettings />,
    },
    // {
    //   id: "integrations",
    //   name: t("Integrations"),
    //   icon: <PlugsConnected weight="light" />,
    //   component: <IntegrationsSettings />,
    // },
  ];

  return (
    <div className="flex flex-1">
      <div className="flex-1 p-8">
        {content.find((c) => c.id === selectedTab)?.component}
      </div>
    </div>
  );
};

export default WorkspaceSettings;
