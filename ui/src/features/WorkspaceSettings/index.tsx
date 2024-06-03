import { PlugsConnected, Toolbox, UsersThree } from "@phosphor-icons/react";
import { useNavigate, useParams } from "@tanstack/react-router";
import { useState } from "react";

import { useT } from "@flow/lib/i18n";

import { TopNavigation } from "../TopNavigation";

import { GeneralSettings, IntegrationsSettings, MembersSettings } from "./components";

type Tab = "general" | "integrations" | "members";

const WorkspaceSettings: React.FC = () => {
  const { workspaceId, tab } = useParams({ strict: false });
  const t = useT();

  const [selectedTab, selectTab] = useState<Tab>(tab ?? "general");

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

  const handleTabChange = (t: Tab) => {
    navigate({ to: `/workspace/${workspaceId}/settings/${t}` });
    selectTab(t);
  };

  return (
    <div className="flex flex-col bg-zinc-800 text-zinc-300 h-[100vh]">
      <TopNavigation />
      <div className="flex-1 flex">
        <div className="flex flex-col bg-zinc-900/50 border-r border-zinc-700 w-[250px] gap-3 px-2 py-4">
          {content.map(({ id, name, icon }) => (
            <div
              key={id}
              className={`flex items-center gap-2 px-2 py-1 rounded cursor-pointer border-l-2 border-transparent hover:bg-zinc-700/50 ${selectedTab === id ? "bg-zinc-700/50 border-red-800/50" : undefined}`}
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
