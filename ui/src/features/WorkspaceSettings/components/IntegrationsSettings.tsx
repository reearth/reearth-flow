import { CaretDown, Plugs } from "@phosphor-icons/react";
import { useState } from "react";

import {
  Button,
  Checkbox,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { useCurrentWorkspace } from "@flow/stores";
import { Role } from "@flow/types";
import { IntegrationMember } from "@flow/types/integration";

type Filter = "all" | Role;

const roles: Role[] = ["OWNER", "MAINTAINER", "READER", "WRITER"];

const IntegrationsSettings: React.FC = () => {
  const t = useT();
  const [currentWorkspace] = useCurrentWorkspace();

  const [currentFilter, setFilter] = useState<Filter>("all");

  const filters: { id: Filter; title: string }[] = [
    { id: "all", title: t("All") },
    { id: "OWNER", title: t("Owner") },
    { id: "READER", title: t("Reader") },
    { id: "MAINTAINER", title: t("Maintainer") },
    { id: "WRITER", title: t("Writer") },
  ];

  const integrations: IntegrationMember[] =
    (currentWorkspace?.members?.filter(
      m =>
        "integration" in m &&
        (currentFilter !== "all" ? m.integrationRole === currentFilter : true),
    ) as IntegrationMember[]) ?? [];

  const [selectedIntegrations, setSelectedIntegrations] = useState<string[]>([]);

  return (
    <div>
      <div className="flex flex-col gap-6 mt-4 max-w-[800px]">
        <div className="flex justify-between">
          <p className="text-lg font-extralight">{t("Integrations Settings")}</p>
          <Button>{t("Connect Integration")}</Button>
        </div>
        <div className="border border-zinc-700 rounded font-extralight">
          <div className="flex justify-between items-center gap-2 p-2 border-b border-zinc-700 h-[42px]">
            <div className="flex items-center gap-2">
              <Checkbox
                className="border-zinc-600 mx-2"
                checked={selectedIntegrations.length === integrations.length}
                onClick={() =>
                  setSelectedIntegrations(
                    selectedIntegrations.length !== integrations.length
                      ? integrations.map(m => m.id)
                      : [],
                  )
                }
              />
              <Plugs weight="thin" />
              <p>
                {selectedIntegrations.length
                  ? `${selectedIntegrations.length} ${selectedIntegrations.length === 1 ? t("member selected") : t("members selected")}`
                  : `${integrations.length} ${t("Integrations")}`}
              </p>
            </div>
            {selectedIntegrations.length > 0 && (
              <div className="flex gap-4">
                <DropdownMenu>
                  <DropdownMenuTrigger className="flex items-center gap-1">
                    <p className="text-sm">{t("Change role")}</p>
                    <CaretDown className="w-2 h-2" />
                  </DropdownMenuTrigger>

                  <DropdownMenuContent className="min-w-[70px]">
                    {roles.map((role, idx) => (
                      <DropdownMenuItem key={idx} onClick={() => console.log(role)}>
                        {role}
                      </DropdownMenuItem>
                    ))}
                  </DropdownMenuContent>
                </DropdownMenu>
                <Button className="h-[25px]" size="sm" variant="destructive">
                  {t("Remove selected")}
                </Button>
              </div>
            )}
            <div>
              <DropdownMenu>
                <DropdownMenuTrigger className="flex items-center gap-2">
                  <p>{filters.find(f => f.id === currentFilter)?.title}</p>
                  <CaretDown className="w-3 h-3" />
                </DropdownMenuTrigger>

                <DropdownMenuContent className="min-w-[70px]">
                  {filters.map((filter, idx) => (
                    <DropdownMenuItem
                      key={idx}
                      className={`justify-center h-[25px] ${filter.id === currentFilter ? "bg-zinc-700/50" : undefined}`}
                      onClick={() => setFilter(filter.id)}>
                      {filter.title}
                    </DropdownMenuItem>
                  ))}
                </DropdownMenuContent>
              </DropdownMenu>
            </div>
          </div>
          <div className="max-h-[50vh] overflow-auto">
            {integrations.map(integration => (
              <div key={integration.id} className="flex items-center gap-4 px-4 py-2">
                <Checkbox
                  className="border-zinc-600"
                  checked={selectedIntegrations.includes(integration.id)}
                  onClick={() =>
                    setSelectedIntegrations(prev =>
                      prev.includes(integration.id)
                        ? [...prev.filter(pm => pm !== integration.id)]
                        : [...prev, integration.id],
                    )
                  }
                />
                <p>{integration.integration?.name}</p>
                <p className="px-4 font-thin capitalize text-sm">{integration.integrationRole}</p>
              </div>
            ))}
          </div>
        </div>
        <Button className="self-end">{t("Save")}</Button>
      </div>
    </div>
  );
};

export { IntegrationsSettings };
