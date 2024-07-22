import { PlugsConnected, Toolbox, UsersThree } from "@phosphor-icons/react";
import { useNavigate, useParams } from "@tanstack/react-router";
import { useState } from "react";

import { TopNavigation } from "@flow/features/TopNavigation";
import { useT } from "@flow/lib/i18n";

import { GeneralSettings, IntegrationsSettings, MembersSettings } from "./components";

type Tab = "general" | "integrations" | "members";

const DEFAULT_TAB: Tab = "general";

const WorkspaceSettings: React.FC = () => {
  const { workspaceId, tab } = useParams({ strict: false });
  const t = useT();
  const navigate = useNavigate();

  const content: { id: Tab; name: string; icon: React.ReactNode; component: React.ReactNode }[] = [
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
    {
      id: "integrations",
      name: t("Integrations"),
      icon: <PlugsConnected weight="light" />,
      component: <IntegrationsSettings />,
    },
  ];
  const checkTab = content.find(c => c.id === tab)?.id;

  const [selectedTab, selectTab] = useState<Tab>(checkTab ?? DEFAULT_TAB);

  const handleTabChange = (t: Tab) => {
    navigate({ to: `/workspace/${workspaceId}/settings/${t}` });
    selectTab(t);
  };

  return (
    <div className="flex h-screen flex-col bg-zinc-800 text-zinc-300">
      <TopNavigation />
      <div className="flex flex-1">
        <div className="flex w-[250px] flex-col gap-3 border-r border-zinc-700 bg-zinc-900/50 px-2 py-4">
          {content.map(({ id, name, icon }) => (
            <div
              key={id}
              className={`flex cursor-pointer items-center gap-2 rounded border-l-2 border-transparent px-2 py-1 hover:bg-background-700/50 ${selectedTab === id ? "border-red-800/50 bg-background-700/50" : undefined}`}
              onClick={() => handleTabChange(id)}>
              {icon}
              <p className="font-extralight">{name}</p>
            </div>
          ))}
        </div>
        <div className="flex-1 p-8">{content.find(c => c.id === selectedTab)?.component}</div>
      </div>
    </div>
  );
};

export { WorkspaceSettings };
